# arena.rs

## Function

The arena (docs/drivesim.md §6): floor, four walls and a roof.

## Interface

- `ARENA_SIDE_M` 4.877 (16 ft), `WALL_H_M` 2.4 (PROVISIONAL), `FLOOR_MU` 1.0.
- `add_arena(world) -> ColliderHandle` (the floor's, for contact
  queries). It also adds the walls and the roof.

## Implementation

- **Source:** the NHRL wiki ("NHRL Facilities", "Cage Design", edited
  2025-08, read 2026-10-09). The 12 lb and 30 lb cages are 16' × 16'
  usable, with a floor of plywood painted with dead-flat paint (not steel),
  double-walled polycarbonate walls on a 6×6 timber skirt, and a roof. The
  wall height isn't published; 2.4 m only has to be taller than any robot
  climbs. The skirt isn't modelled separately: the walls are a rigid
  boundary from the floor up.

**The roof** (user, 2026-10-10: "add a roof so its impossible for bots
to launch themselves out of the box") is a 10 cm slab resting on the
walls' tops at `WALL_H_M`, overhanging them like the floor's corners, so
there is no seam to slip through. Before it, a hit could throw a robot
over the 2.4 m walls and out of the world. At 4 kHz even 30 m/s moves
7.5 mm a step, well under its 10 cm, so nothing tunnels through.

The floor's friction is 1.0 and combines by multiplying, so each tyre
grips with exactly its robot.toml `mu`. Rapier uses the stronger of two
colliders' combine rules, so every robot collider sets Multiply too.

There used to be a target box here: first a fixed 5.44 kg, then as heavy
as the robot. On 2026-10-10 the user replaced it with an opponent copy
of the robot (`sim.rs`), then brought it back the same day as the
alternative opponent, in its own file (`target.rs`).

## Assertions

- The arena is centred on the origin, with the floor top at z = 0.
- Closed on all six sides: floor, four walls and roof overlap at every
  edge (tests/arena.rs throws a robot at the roof and a top corner).
