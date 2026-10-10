# loop.py

## Function
The agent loop: one user message in, model turns and tool calls until the
model stops. Adapted from the Combat Robot Analyzer (2026-10-09), with the
solve queue and sandbox removed.

## Interface
- `Agent(store, model, settings, bus)`.
- `robot_dir(robot_id) -> Path`: creates the robot's directory, mode 0777,
  because the import sandbox runs as nobody.
- `async notify(robot_id, text)`: wakes the agent with an "[Automated notice
  — not from the user]" message. Used when an import finishes.
- `async run(robot_id, user_text)`: one turn sequence. Exceptions become
  `error` events, and an `idle` event always follows.
- `MAX_STEPS` 40, `MAX_MALFORMED` 2, `MAX_RESULT_CHARS` 30,000.
- Each model turn's usage is recorded against the robot owner's daily
  budget (`Store.record_usage`, weighted by `usage.weighted` for
  `settings.model`). The budget is enforced in `app.py`, before a
  message, never mid-reply.

## Implementation
- **The history is append-only,** stored and replayed exactly as the API
  returned it, so thinking blocks stay valid.
- **One lock per robot** serialises its turns.
- **Interrupted tools:** a `tool_use` left unanswered (the server restarted
  mid-tool) gets an error `tool_result` appended (`_close_dangling`), never
  removed.
- **Tool calls:** each one validates its input with `tools.validate` and runs
  with a `Ctx`. A `ToolError` or a bug comes back to the model as an
  `is_error` result, not a crash.
- **Refusal:** a declined turn is discarded and reported as an error event.
- **Pause:** `pause_turn` continues the loop.
- **Malformed tool input:** the turn is re-issued, up to `MAX_MALFORMED`
  times.

## Assertions
- [ ] No stored message is ever edited or removed.
- [ ] Every `tool_use` in a stored assistant turn gets exactly one
  `tool_result` in the next user turn.
