# access.py

## Function
Who may touch a robot (docs/drivesim.md §3): the FastAPI dependencies
every `/api/robots/{id}...` route resolves its id through. Moved out of
`app.py` (2026-10-10) when public robots added a third level.

## Interface
Each takes the path's `robot_id` and the caller (`auth.current_user`), and
returns the id or raises 404 (never 403, so ids don't leak):
- `owned`: the caller owns it. For changing a robot or spending its
  owner's budget.
- `viewable`: owned, or the caller is an admin (user, 2026-10-10). For
  reading it: its page, conversation and events.
- `drivable`: viewable, or the robot is public (user, 2026-10-10). For
  its bundle only.

## Implementation
- An id that isn't 32 characters is refused before touching the store.
- A public robot's conversation stays private: only the bundle route
  uses `drivable`, so publishing shares the 3D model and robot file, the
  part of a robot the sim needs, and nothing else.
- House robots (`house.py`) belong to `drivesim-house`, which no
  authentik login is, so they are drivable (public) but never
  viewable or owned.

## Assertions
- [ ] Each level includes the one above it: owned ⊂ viewable ⊂ drivable.
- [ ] Refusal is always 404 with the same detail.
