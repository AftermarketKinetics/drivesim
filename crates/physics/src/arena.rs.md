# arena.rs

## Function

The arena (docs/drivesim.md §6): floor, four walls and the free target
box.

## Interface

- `ARENA_SIDE_M` 4.88 (PROVISIONAL: 16 ft, until the NHRL cage size is
  known), `WALL_H_M` 0.6, `FLOOR_MU` 1.0.
- `BOX_MASS_KG` 5.44, `BOX_HALF_M` [0.15, 0.15, 0.075], `BOX_MU` 0.4.
- `add_arena(world)`; `add_target(world, x, y) -> RigidBodyHandle`.

## Implementation

The floor's friction is 1.0 and combines by multiplying, so each tyre
grips with exactly its robot.toml `mu`. Rapier uses the stronger of two
colliders' combine rules, so every robot and box collider sets Multiply
too.

## Assertions

- The arena is centred on the origin, with the floor top at z = 0.
