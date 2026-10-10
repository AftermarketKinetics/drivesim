# arena.rs (tests)

## Function

The arena is closed: a robot thrown hard at the roof, or at a top
corner where roof and two walls meet, stays inside (user, 2026-10-10:
"add a roof so its impossible for bots to launch themselves out of the
box").

## Interface

Tests:
- `a_robot_thrown_straight_up_stays_under_the_roof`: every body of the
  fixture at 30 m/s up (46 m with no roof), 3 s: the chassis stays
  under `WALL_H_M` and over the floor.
- `a_robot_thrown_at_a_top_corner_stays_inside`: (−15, 15, 15) m/s at
  the −x/+y top corner, 3 s: same bounds.

## Implementation

The throw sets every body's velocity, not the chassis alone, so the
wheels and weapon don't drag the chassis back through their joints. It
uses the box opponent, which sits at (1, 0), out of both throws' way.
The chassis centre is checked, against the walls' and roof's inner
faces.

## Assertions

- Bounds come from `arena.rs` constants, never from a measured run.
