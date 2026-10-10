# target_render.rs

## Function

Draws the box opponent (physics `target.rs`) where the physics puts it.

## Interface

- `spawn_target_visual(commands, root, handle, color, meshes, mats)`: a
  cuboid of `BOX_HALF_M`'s size in `color`, a child of `root`, linked to
  `handle` with `BodyLink` so `sync_bodies` moves it.

## Implementation

- The size comes from `drivesim_physics::target::BOX_HALF_M`, so what is
  drawn is what collides.
- It is the removed `arena_render::spawn_target`, restored on its own;
  `round.rs` passes `OPPONENT_ORANGE`.

## Assertions

- The visual carries `BodyLink`, so `round::build_round` despawns it on
  the next switch.
