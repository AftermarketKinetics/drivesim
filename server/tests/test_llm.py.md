# test_llm.py

## Function
Checks `AnthropicClient`'s request shape and drop logging against a fake SDK
client. There is no network and no cost.

## Interface
Tests:
- `test_stale_thinking_blocks_are_dropped_not_fatal`: the request sends the
  `drop_block` thinking binding and both betas, and returns content
  unchanged.
- `test_dropped_blocks_are_logged`: a `thinking_dropped` entry is logged
  with its path and reason; unknown entry types are ignored.

## Implementation
The fake replaces `client.beta.messages.stream` with a recorder that returns
an async context manager. It yields no events and gives a final message
carrying `input_transformations`. `AnthropicClient` is built with `__new__`,
so no real SDK client (and no credentials) is needed.

## Assertions
- Never touches the network.
