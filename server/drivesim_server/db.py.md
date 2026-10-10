# db.py

## Function
SQLite store for robots and their append-only role-conversation history.
Adapted from the Combat Robot Analyzer's session store (2026-10-09); robots
persist until deleted, unlike the analyzer's expiring sessions.

## Interface
- `STATUSES`: `importing`, `ready`, `failed`.
- `WEIGHT_CLASSES`: (3, 12, 30), NHRL's classes in lb.
- `MIGRATIONS`: `(table, column, definition)` added after the first
  deploy; `Store()` adds any a file lacks.
- `Store(path)` with these methods:
  - **Creating and listing:** `create_robot(username, name) -> id` (32 hex characters, status `importing`); `robots_for(username)`, newest first; `robot(id)`; `owner(id)`.
  - **Updating:** `set_status(id, status, error="")`; `set_checked(id, checked, name=None)`.
  - **History:** `append(id, role, content)`; `messages(id) -> [{"role", "content"}]`.
  - **Deleting:** `delete_robot(id)`.
  - **Usage:** `record_usage(username, robot_id, model, usage, weighted)`; `used_since(username, since) -> float` (weighted sum since an epoch time); `used_by_user_since(since) -> {username: float}`.
  - **Public** (user, 2026-10-10): `set_public(id, public, weight_lb)`
    (public needs a class in `WEIGHT_CLASSES`; private keeps its class
    or 0); `public_robots()`: public, ready and checked robots, `{id,
    name, username, weight_lb}`, by class then name (case-insensitive);
    `seed_robot(id, username, name, weight_lb)`: insert a ready, checked,
    public robot with a fixed id if it is absent (`house.py`).
  - **Admin:** `all_robots()`: every robot, newest first, with `username` and `used` (its lifetime weighted usage).
- Columns of `robots`: `id`, `username`, `name`, `status`, `error`, `checked`, `created`, `public` (0/1), `weight_lb` (0 = untagged).
- Columns of `usage`: `username`, `robot_id`, `at`, `model`, `usage` (the turn's JSON), `weighted`. It does not reference `robots`: deleting a robot must not hand its usage back to the daily budget.

## Implementation
- **Content is stored exactly as the API returned it,** as JSON, and replayed unchanged. Thinking blocks are only valid in the conversation that produced them.
- **`checked`** is 1 when the last `robot.toml` write passed `drivesim-check`; that's what "ready to drive" means.
- One connection per call; WAL is not needed at this load.
- **Migrations are additive `ALTER TABLE ADD COLUMN`s,** checked against
  `PRAGMA table_info` at every start, because `CREATE TABLE IF NOT EXISTS`
  never changes an existing table. accelerate's database predates
  `public`/`weight_lb` (2026-10-10).
- **`public_robots` re-checks `ready` and `checked`,** so a public robot
  whose later robot.toml fails the check drops out of the opponent list
  without losing its flag.
- Slightly over ~200 lines: one class over one SQLite file, with no
  natural seam to split along.

## Assertions
- [ ] No message is ever updated or deleted except by `delete_robot` (all of them).
- [ ] `set_status` rejects anything outside `STATUSES`.
- [ ] A robot is never public without a weight class in `WEIGHT_CLASSES`.
- [ ] Every column in `SCHEMA` added after the first deploy is also in `MIGRATIONS`.
