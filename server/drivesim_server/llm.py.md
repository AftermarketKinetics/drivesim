# llm.py

## Function
The model behind the agent, behind one small interface. The loop only ever
calls `ModelClient.turn`: send the history, stream text out as it arrives, get
back the assistant turn as plain JSON content blocks. Today that is Claude
through the Anthropic SDK; a local model later is a second implementation of
the same method, and nothing else changes. Copied unchanged from the Combat
Robot Analyzer (2026-10-09); drivesim runs it on `claude-sonnet-5-5` (user,
2026-10-09; `DRIVESIM_MODEL`).

## Interface
- `OnText = Callable[[str], Awaitable[None]]`.
- `Turn` (frozen): `content: list[dict]` (blocks exactly as returned, JSON-serialisable), `stop_reason: str | None`, `usage: dict`.
- `ModelClient` (Protocol): `async turn(*, system, tools, messages, on_text) -> Turn`.
- `MalformedToolInput(Exception)` — the model emitted tool-input JSON the SDK could not parse at all; the caller re-issues the turn.
- `AnthropicClient(model, effort)` — implements `ModelClient`.
- `THINKING`, `BETAS`: the thinking configuration and beta list every
  request sends.
- `log_dropped(message)`: logs each `thinking_dropped` entry in a
  response's `input_transformations` as a warning.

## Implementation
- Streams with `client.beta.messages.stream`, `max_tokens=64000`, `output_config={"effort": effort}`.
- Every tool gets `eager_input_streaming: True`; because streamed tool input is not validated by the API, `tools/base.validate` checks it before a tool runs.
- Top-level `cache_control={"type": "ephemeral"}` caches tools + system + history prefix.
- `betas=BETAS`, `fallbacks="default"`: `server-side-fallback-2026-07-01` retries a classifier decline server-side on the model Anthropic recommends for that category.
- **Stale thinking is dropped, not fatal.** Each request sends
  `thinking={"type": "adaptive", "block_binding": {"prefix_mismatch_behavior":
  "drop_block"}}` under the beta `thinking-binding-controls-2026-08-01`. The
  field without that beta is itself a 400.
  - Why: a stored thinking block is bound to the system prompt and tools it
    was made under. On 2026-10-10 a prompt and tool-description change
    failed the next turn of an open conversation with "Invalid `signature`
    in `thinking` block … bound to a different conversation", and every
    deploy that touches either would do the same to every open
    conversation.
  - Now the API removes the stale blocks, reports each in
    `input_transformations` (logged by `log_dropped`), and the conversation
    carries on without that reasoning.
  - The shape was checked against the real API through `count_tokens` with
    drivesim's prompt on claude-sonnet-5-5 (accepted, 2026-10-10).
- A `ValueError` from the stream iterator means unparseable eager tool input: there is no complete `tool_use` block to answer, so it becomes `MalformedToolInput`. API errors are not `ValueError` and propagate.
- Blocks are dumped with `model_dump(mode="json", exclude_none=True)` so they replay unchanged from SQLite.

## Assertions
- [ ] `loop.py` depends only on `ModelClient`, `Turn` and `MalformedToolInput`, never on `anthropic`.
- [ ] `Turn.content` is returned unmodified (no filtering of thinking blocks).
- [ ] Every request carries `THINKING` and both `BETAS` (tests/test_llm.py).
- [ ] The API key is read by the SDK from the service's environment and never passed to a tool or the import sandbox. drivesim shares the analyzer's key on accelerate (user, 2026-10-09).
