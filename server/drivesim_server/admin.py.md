# admin.py

## Function
The admin dashboard's API (user, 2026-10-10): lets the deployment's
admins see every robot, who owns it, and what it has cost, so problems
reported to arctic256 on Discord can be found by robot ID.

## Interface
- `router`: a FastAPI `APIRouter`, included by `app.py` before the static
  mount.
- `GET /api/admin/robots` (needs `auth.admin_user`, 403 otherwise) returns:
  - `robots`: every robot, newest first, as `id, username, name, status,
    error, checked, created, used` (`used` is the robot's lifetime
    weighted usage, rounded);
  - `users_today`: `{username: weighted usage since midnight}`, only for
    users with usage today;
  - `budget`: the per-user daily budget;
  - `resets_at`: when today's usage resets (ISO 8601).

## Implementation
- **Read only.** Opening, reading and driving another user's robot goes
  through the ordinary robot routes, which let admins in via `viewable`.
  Nothing here changes a robot.
- Its own router, not more routes in `app.py`, so the admin surface is
  one file. It reads state from `request.app.state.s` directly rather
  than importing `app.py`, which would be circular.
- "Today" uses `usage.day_bounds` with `budget_tz`, the same day the
  budget is counted over.

## Assertions
- [ ] Every route here depends on `admin_user`.
- [ ] Nothing here writes to the store or the filesystem.
