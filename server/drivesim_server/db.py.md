# db.py

## Function
SQLite store for robots and their append-only role-conversation history.
Adapted from the Combat Robot Analyzer's session store (2026-10-09); robots
persist until deleted, unlike the analyzer's expiring sessions.

## Interface
- `STATUSES`: `importing`, `ready`, `failed`.
- `Store(path)` with these methods:
  - **Creating and listing:** `create_robot(username, name) -> id` (32 hex characters, status `importing`); `robots_for(username)`, newest first; `robot(id)`; `owner(id)`.
  - **Updating:** `set_status(id, status, error="")`; `set_checked(id, checked, name=None)`.
  - **History:** `append(id, role, content)`; `messages(id) -> [{"role", "content"}]`.
  - **Deleting:** `delete_robot(id)`.
  - **Usage:** `record_usage(username, robot_id, model, usage, weighted)`; `used_since(username, since) -> float` (weighted sum since an epoch time).
- Columns of `robots`: `id`, `username`, `name`, `status`, `error`, `checked`, `created`.
- Columns of `usage`: `username`, `robot_id`, `at`, `model`, `usage` (the turn's JSON), `weighted`. It does not reference `robots`: deleting a robot must not hand its usage back to the daily budget.

## Implementation
- **Content is stored exactly as the API returned it,** as JSON, and replayed unchanged. Thinking blocks are only valid in the conversation that produced them.
- **`checked`** is 1 when the last `robot.toml` write passed `drivesim-check`; that's what "ready to drive" means.
- One connection per call; WAL is not needed at this load.

## Assertions
- [ ] No message is ever updated or deleted except by `delete_robot` (all of them).
- [ ] `set_status` rejects anything outside `STATUSES`.
