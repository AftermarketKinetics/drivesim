# events.py

## Function
Per-robot event fan-out to whoever is watching (SSE connections from
`robot.html`). Copied from the Combat Robot Analyzer (2026-10-09). Events are
best-effort and not stored: the message history in SQLite is the record, and
a client that reconnects reloads it.

## Interface
- `EventBus()`
  - `subscribe(robot_id) -> asyncio.Queue[dict]` (bounded, 1000).
  - `unsubscribe(robot_id, q)`: drops the robot's entry once its last subscriber leaves.
  - `async emit(robot_id, event: dict)`: puts the event on every subscriber's queue.
- Event types emitted elsewhere (all have `type`):
  - `loop.py`: `text{text}`, `tool_start{name, input}`, `tool_end{name, is_error}`, `turn_end{stop_reason, usage}`, `error{message}`, `idle`.
  - `tools/write_robot_toml.py`: `robot{report}` (the checker's JSON).
  - `app.py`: `status{status: "ready", parts}` or `status{status: "failed", error}` when an import ends.

## Implementation
- `emit` uses `put_nowait` and drops the event on a full queue: a stalled client misses events rather than stalling the agent.
- In-process only; one service process is assumed.

## Assertions
- [ ] `emit` never blocks or raises on a slow subscriber.
- [ ] Nothing reads events back as state; any state a client needs is also in SQLite or the robot's directory.
- [ ] Events carry no secret (the API key, other users' data).
