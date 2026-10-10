# arena_render.rs

## Function

Draws NHRL's 12 lb cage (physics `arena.rs`): the painted plywood floor,
see-through polycarbonate walls, the light, and the target box.

## Interface

- `spawn_arena(commands, root, meshes, mats)`: floor, four walls and a
  shadow-casting directional light, all under `root`.
- `spawn_target(commands, root, handle, meshes, mats)`: the box, drawn
  through `BodyLink` so `sync_bodies` moves it.

## Implementation

- Sizes come from physics `arena.rs` (`ARENA_SIDE_M`, `WALL_H_M`,
  `BOX_HALF_M`). What is drawn matches what is simulated.
- The floor's top face is at z = 0, the physics floor's surface.
- Walls are alpha-blended at 0.12 so the driver can see through them, as
  through real polycarbonate, and sit just outside the usable 16 ft
  square.

## Assertions

- The drawn floor, walls and box use the physics constants, never their
  own numbers.
