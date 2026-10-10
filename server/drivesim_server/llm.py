"""The model behind the agent, behind one small interface.

The loop only ever calls ``ModelClient.turn``: send the history, stream text
out as it arrives, get back the assistant turn as plain JSON content blocks.
Today that is Claude through the Anthropic SDK; a local model later is a
second implementation of the same method, and nothing else changes.
Copied from the Combat Robot Analyzer (2026-10-09).
"""

from __future__ import annotations

import logging
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any, Protocol

import anthropic

OnText = Callable[[str], Awaitable[None]]

log = logging.getLogger(__name__)

#: Thinking blocks in a stored conversation are bound to the system prompt
#: and tools they were made under. Changing either (every deploy that edits
#: the prompt or a tool description) used to fail the next turn of every open
#: conversation with a 400 (2026-10-10). With this, the API drops the stale
#: blocks instead, and the conversation carries on without that reasoning.
THINKING = {"type": "adaptive", "block_binding": {"prefix_mismatch_behavior": "drop_block"}}
BETAS = [
    # A classifier decline is retried server-side on the model Anthropic
    # recommends for that category.
    "server-side-fallback-2026-07-01",
    # Required for THINKING's block_binding; without it that field is a 400.
    "thinking-binding-controls-2026-08-01",
]


@dataclass(frozen=True)
class Turn:
    #: Content blocks exactly as returned, JSON-serialisable, replayed unchanged.
    content: list[dict[str, Any]]
    stop_reason: str | None
    usage: dict[str, Any]


class ModelClient(Protocol):
    async def turn(
        self,
        *,
        system: str,
        tools: list[dict[str, Any]],
        messages: list[dict[str, Any]],
        on_text: OnText,
    ) -> Turn: ...


class MalformedToolInput(Exception):
    """The model emitted tool-input JSON the SDK could not parse at all."""


def log_dropped(message: Any) -> None:
    """Log each thinking block the API dropped from this request (the
    response's `input_transformations`), so lost reasoning is visible."""
    for t in getattr(message, "input_transformations", None) or []:
        t = t if isinstance(t, dict) else t.model_dump(mode="json", exclude_none=True)
        if t.get("type") == "thinking_dropped":
            log.warning("thinking block dropped: %s (%s)", t.get("path"), t.get("reason"))


class AnthropicClient:
    """Claude, streamed, with server-side refusal fallback."""

    def __init__(self, model: str, effort: str) -> None:
        self.client = anthropic.AsyncAnthropic()
        self.model = model
        self.effort = effort

    async def turn(
        self,
        *,
        system: str,
        tools: list[dict[str, Any]],
        messages: list[dict[str, Any]],
        on_text: OnText,
    ) -> Turn:
        try:
            async with self.client.beta.messages.stream(
                model=self.model,
                max_tokens=64000,
                system=system,
                tools=[{**t, "eager_input_streaming": True} for t in tools],
                messages=messages,
                output_config={"effort": self.effort},
                thinking=THINKING,
                # Top-level auto caching: tools + system + history prefix.
                cache_control={"type": "ephemeral"},
                betas=BETAS,
                fallbacks="default",
            ) as stream:
                async for event in stream:
                    if event.type == "text":
                        await on_text(event.text)
                final = await stream.get_final_message()
        except ValueError as exc:
            # Raised from the stream iterator for unparseable eager tool input;
            # there is no complete tool_use block to answer, so the turn is
            # re-issued by the caller. API errors are not ValueError.
            raise MalformedToolInput(str(exc)) from exc
        log_dropped(final)
        return Turn(
            content=[b.model_dump(mode="json", exclude_none=True) for b in final.content],
            stop_reason=final.stop_reason,
            usage=final.usage.model_dump(mode="json", exclude_none=True),
        )
