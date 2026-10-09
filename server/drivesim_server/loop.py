"""The agent loop: one user message in, model turns and tool calls until done.

Adapted from the Combat Robot Analyzer (2026-10-09). History is append-only —
every turn is stored exactly as the API returned it and replayed unchanged —
so thinking blocks stay valid. Turns for one robot are serialised by a lock;
the import's completion wakes the agent with an automated notice through the
same entry point.
"""

from __future__ import annotations

import asyncio
import json
from collections import defaultdict
from pathlib import Path
from typing import Any

from .config import Settings
from .db import Store
from .events import EventBus
from .llm import MalformedToolInput, ModelClient
from .prompt import SYSTEM
from .tools import TOOLS, ToolError, validate
from .tools.base import Ctx

MAX_STEPS = 40
MAX_MALFORMED = 2
#: Tool results bigger than this are cut, so one verbose tool cannot flood context.
MAX_RESULT_CHARS = 30_000


class Agent:
    def __init__(self, store: Store, model: ModelClient, settings: Settings, bus: EventBus) -> None:
        self.store = store
        self.model = model
        self.settings = settings
        self.bus = bus
        self._locks: dict[str, asyncio.Lock] = defaultdict(asyncio.Lock)
        self._tool_defs = [t.definition() for t in TOOLS.values()]

    def robot_dir(self, robot_id: str) -> Path:
        d = self.settings.robots_root / robot_id
        d.mkdir(parents=True, exist_ok=True)
        d.chmod(0o777)  # the import sandbox runs as nobody and writes bundle/ here
        return d

    async def notify(self, robot_id: str, text: str) -> None:
        """Wake the agent with an automated notice (not from the user)."""
        await self.run(robot_id, f"[Automated notice — not from the user] {text}")

    async def run(self, robot_id: str, user_text: str) -> None:
        async with self._locks[robot_id]:
            try:
                await self._run(robot_id, user_text)
            except Exception as exc:
                await self.bus.emit(robot_id, {"type": "error", "message": f"{type(exc).__name__}: {exc}"})
            finally:
                await self.bus.emit(robot_id, {"type": "idle"})

    async def _run(self, robot_id: str, user_text: str) -> None:
        emit = lambda ev: self.bus.emit(robot_id, ev)  # noqa: E731
        ctx = Ctx(robot_id, self.robot_dir(robot_id), self.settings, self.store, emit)
        self._close_dangling(robot_id)
        self.store.append(robot_id, "user", [{"type": "text", "text": user_text}])
        messages = self.store.messages(robot_id)
        malformed = 0

        for _ in range(MAX_STEPS):
            async def on_text(t: str) -> None:
                await emit({"type": "text", "text": t})

            try:
                turn = await self.model.turn(system=SYSTEM, tools=self._tool_defs,
                                             messages=messages, on_text=on_text)
            except MalformedToolInput:
                malformed += 1
                if malformed > MAX_MALFORMED:
                    raise
                continue
            malformed = 0

            if turn.stop_reason == "refusal":
                # Discard any partial output; the history stays as it was.
                await emit({"type": "error", "message": "The model declined this request."})
                return

            self.store.append(robot_id, "assistant", turn.content)
            messages.append({"role": "assistant", "content": turn.content})
            await emit({"type": "turn_end", "stop_reason": turn.stop_reason, "usage": turn.usage})

            if turn.stop_reason == "pause_turn":
                continue
            tool_uses = [b for b in turn.content if b.get("type") == "tool_use"]
            if not tool_uses:
                return

            results = []
            for block in tool_uses:
                results.append(await self._call(ctx, block, truncated=turn.stop_reason == "max_tokens"))
            self.store.append(robot_id, "user", results)
            messages.append({"role": "user", "content": results})

        await emit({"type": "error", "message": f"stopped after {MAX_STEPS} steps"})

    def _close_dangling(self, robot_id: str) -> None:
        """Answer tool calls left unanswered when the service stopped mid-tool.

        The API rejects a history whose last assistant turn has tool_use blocks
        with no matching tool_result, and the history is append-only, so the
        repair is to append the missing results — never to drop the turn.
        """
        history = self.store.messages(robot_id)
        if not history or history[-1]["role"] != "assistant":
            return
        pending = [b for b in history[-1]["content"] if b.get("type") == "tool_use"]
        if pending:
            self.store.append(robot_id, "user", [
                {"type": "tool_result", "tool_use_id": b["id"], "is_error": True,
                 "content": "interrupted: the server restarted before this tool finished"}
                for b in pending])

    async def _call(self, ctx: Ctx, block: dict[str, Any], *, truncated: bool) -> dict[str, Any]:
        name, args = block.get("name"), block.get("input")
        await self.bus.emit(ctx.robot_id, {"type": "tool_start", "name": name, "input": args})
        try:
            if truncated:
                raise ToolError("tool input was cut off by the output limit; call it again")
            tool = TOOLS.get(name)
            if tool is None:
                raise ToolError(f"unknown tool {name!r}")
            validate(tool.input_schema, args)
            out = await tool.run(ctx, args)
            text, is_error = json.dumps(out, default=str), False
        except ToolError as exc:
            text, is_error = str(exc), True
        except Exception as exc:  # a tool bug reaches the model as an error, not a crash
            text, is_error = f"internal error: {type(exc).__name__}: {exc}", True
        if len(text) > MAX_RESULT_CHARS:
            text = text[:MAX_RESULT_CHARS] + " …[truncated]"
        await self.bus.emit(ctx.robot_id, {"type": "tool_end", "name": name, "is_error": is_error})
        result: dict[str, Any] = {"type": "tool_result", "tool_use_id": block["id"], "content": text}
        if is_error:
            result["is_error"] = True
        return result
