# arena_render.rs

## Function

Draws NHRL's 12 lb cage (physics `arena.rs`): the painted plywood floor,
see-through polycarbonate walls, overhead lights, and the target box.

## Interface

- `spawn_arena(commands, root, meshes, mats, images)`: floor (textured by
  `floor_texture.rs`), four walls and the overhead lights, all under
  `root`.
- `LIGHT_HEIGHT_M` = 3.5.
- `spawn_target(commands, root, handle, meshes, mats)`: the box, drawn
  through `BodyLink` so `sync_bodies` moves it.

## Implementation

- Sizes come from physics `arena.rs` (`ARENA_SIDE_M`, `WALL_H_M`,
  `BOX_HALF_M`). What is drawn matches what is simulated.
- The floor's top face is at z = 0, the physics floor's surface.
- Walls are alpha-blended at 0.12 so the driver can see through them, as
  through real polycarbonate, and sit just outside the usable 16 ft
  square.

- **Lighting is overhead, as at an event, and casts no shadows** (user,
  2026-10-10). It replaced the first build's single shadow-mapped sun, and has three
  parts:
  - **a downward directional "truss" light**, 6,000 lx: many lamps
    together act like even light from straight above;
  - **a 3 × 3 grid of point lamps**, 3.5 m up above the cage roof,
    600,000 lm each, each with an unlit emissive panel so it can be
    seen;
  - **global ambient fill** at 600 cd/m². From the driver's eye most of
    the robot and box is vertical faces, which overhead light only
    grazes.

  The first try was the lamps alone at 150,000 lm, about 1,000 lx under
  each, and it was "really dark" (user, 2026-10-10).

## Assertions

- The drawn floor, walls and box use the physics constants, never their
  own numbers.
- No light in the scene has `shadow_maps_enabled`.
