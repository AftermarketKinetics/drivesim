# arena.rs

## Function

The arena (docs/drivesim.md §6): floor, four walls and the free target
box.

## Interface

- `ARENA_SIDE_M` 4.877 (16 ft), `WALL_H_M` 2.4 (PROVISIONAL), `FLOOR_MU` 1.0.
- `BOX_HALF_M` [0.15, 0.15, 0.075], `BOX_MU` 0.4.
- `add_arena(world) -> ColliderHandle` (the floor's, for contact
  queries); `add_target(world, x, y, mass_kg) -> RigidBodyHandle`.

## Implementation

- **Source:** the NHRL wiki ("NHRL Facilities", "Cage Design", edited
  2025-08, read 2026-10-09). The 12 lb and 30 lb cages are 16' × 16'
  usable, with a floor of plywood painted with dead-flat paint (not steel),
  double-walled polycarbonate walls on a 6×6 timber skirt, and a roof. The
  wall height isn't published; 2.4 m only has to be taller than any robot
  climbs. The skirt isn't modelled separately: the walls are a rigid
  boundary from the floor up.

The floor's friction is 1.0 and combines by multiplying, so each tyre
grips with exactly its robot.toml `mu`. Rapier uses the stronger of two
colliders' combine rules, so every robot and box collider sets Multiply
too.

The box weighs what the robot does (user, 2026-10-10; `Sim::new` passes
the model's total mass). Before that it was a fixed 5.44 kg, the 12 lb
limit. A fixed box would be too heavy for a lighter robot and too light
for a 30 lb one.

## Assertions

- The arena is centred on the origin, with the floor top at z = 0.
