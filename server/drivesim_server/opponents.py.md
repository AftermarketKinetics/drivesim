# opponents.py

## Function
Public robots as AI opponents (user, 2026-10-10): the owner tags a robot
public with its weight class, and every signed-in user's sim page lists
the public robots for its AI to drive (docs/drivesim.md §3, §8).

## Interface
A router `app.py` includes:

| Route | Does |
|---|---|
| `PUT /api/robots/{id}/public` `{"public", "weight_lb"}` | owner only (`owned`); 200 `{"public", "weight_lb"}`. 422 unless `weight_lb` is in `WEIGHT_CLASSES` (or 0 when making it private); 409 when making public a robot that isn't ready and checked |
| `GET /api/opponents` | every public, drivable robot: `[{"id", "name", "weight_lb", "mine"}]`, by class then name |

## Implementation
- **The owner picks the class** (user, 2026-10-10), not the mass: a robot
  under its class limit still fights in that class, and the computed
  mass can be wrong.
- **Owners' usernames are not listed.** The page shows names and classes
  only; `mine` marks the caller's own.
- Only a drivable robot can be made public, and the list re-checks it
  (`Store.public_robots`), so a broken robot is never offered.
- `PUT` because the body is the whole public state, not a change to it.

## Assertions
- [ ] Only the owner can change a robot's public state (not an admin).
- [ ] `GET /api/opponents` never returns a robot that isn't public,
  ready and checked.
