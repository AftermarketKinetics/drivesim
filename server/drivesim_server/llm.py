"""The model behind the agent, behind one small interface.

The loop only ever calls ``ModelClient.turn``: send the history, stream text
out as it arrives, get back the assistant turn as plain JSON content blocks.
Today that is Claude through the Anthropic SDK; a local model later is a
second implementation of the same method, and nothing else changes.
Copied unchanged from the Combat Robot Analyzer (2026-10-09).
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import Any, Protocol

import anthropic

OnText = Callable[[str], Awaitable[None]]


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
                # Top-level auto caching: tools + system + history prefix.
                cache_control={"type": "ephemeral"},
                # A classifier decline is retried server-side on the model
                # Anthropic recommends for that category.
                betas=["server-side-fallback-2026-07-01"],
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
        return Turn(
            content=[b.model_dump(mode="json", exclude_none=True) for b in final.content],
            stop_reason=final.stop_reason,
            usage=final.usage.model_dump(mode="json", exclude_none=True),
        )
