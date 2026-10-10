"""AnthropicClient's request shape, against a fake SDK (no network)."""

from __future__ import annotations

import logging
from types import SimpleNamespace

from drivesim_server import llm


class FakeStream:
    def __init__(self, final):
        self.final = final

    async def __aenter__(self):
        return self

    async def __aexit__(self, *exc):
        return False

    def __aiter__(self):
        return self

    async def __anext__(self):
        raise StopAsyncIteration

    async def get_final_message(self):
        return self.final


def fake_client(sent, transformations):
    block = SimpleNamespace(model_dump=lambda **_: {"type": "text", "text": "hi"})
    usage = SimpleNamespace(model_dump=lambda **_: {"input_tokens": 1})
    final = SimpleNamespace(content=[block], stop_reason="end_turn", usage=usage,
                            input_transformations=transformations)

    def stream(**kwargs):
        sent.update(kwargs)
        return FakeStream(final)

    return SimpleNamespace(beta=SimpleNamespace(messages=SimpleNamespace(stream=stream)))


async def run_turn(transformations):
    sent: dict = {}
    client = llm.AnthropicClient.__new__(llm.AnthropicClient)
    client.client, client.model, client.effort = fake_client(sent, transformations), "m", "medium"

    async def on_text(_):
        pass

    turn = await client.turn(system="s", tools=[], messages=[], on_text=on_text)
    return sent, turn


async def test_stale_thinking_blocks_are_dropped_not_fatal():
    sent, turn = await run_turn([])
    assert sent["thinking"] == {"type": "adaptive",
                                "block_binding": {"prefix_mismatch_behavior": "drop_block"}}
    # The block_binding field is a 400 without its beta.
    assert "thinking-binding-controls-2026-08-01" in sent["betas"]
    assert "server-side-fallback-2026-07-01" in sent["betas"]
    assert turn.content == [{"type": "text", "text": "hi"}]


async def test_dropped_blocks_are_logged(caplog):
    dropped = {"type": "thinking_dropped", "path": "messages.1.content.6",
               "reason": "prefix_binding_mismatch"}
    with caplog.at_level(logging.WARNING, logger="drivesim_server.llm"):
        await run_turn([dropped, {"type": "something_new"}])
    assert "messages.1.content.6" in caplog.text and "prefix_binding_mismatch" in caplog.text
    assert caplog.text.count("dropped") == 1
