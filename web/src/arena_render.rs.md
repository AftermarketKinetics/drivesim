# arena_render.rs

## Function

Draws NHRL's 12 lb cage (physics `arena.rs`): the painted plywood floor,
see-through polycarbonate walls with distance markings, a see-through
roof, a dark rail along the walls' tops, and overhead lights.

## Interface

- `spawn_arena(commands, root, meshes, mats, images)`: floor (textured by
  `floor_texture.rs`), four walls with their distance markings
  (`wall_markings.rs`), the roof, the top rail and the overhead
  lights, all under `root`.
- `RAIL_M` = 0.04, `LIGHT_HEIGHT_M` = 3.5.

## Implementation

- Sizes come from physics `arena.rs` (`ARENA_SIDE_M`, `WALL_H_M`). What is drawn matches what is simulated.
- The floor's top face is at z = 0, the physics floor's surface.
- Walls are alpha-blended at 0.12 so the driver can see through them, as
  through real polycarbonate, and sit just outside the usable 16 ft
  square.
- The roof (user, 2026-10-10) uses the same material, on the walls' tops
  at `WALL_H_M`. The driver's eye (1.7 m) is below it, so it is seen
  only at a grazing angle. With no shadow maps it doesn't dim the
  lamps above it.
- **The top rail** (user, 2026-10-10): a 4 cm opaque dark-grey bar
  along each wall's top edge, just under the roof. Without it the upper
  1.4 m of wall is a featureless tint with no visible end, and a robot
  thrown over a wall (before the roof) looked like it went through it.
  Drawn only: the physics walls and roof already close the cage.

- **Lighting is overhead, as at an event, and casts no shadows** (user,
  2026-10-10). It replaced the first build's single shadow-mapped sun, and has three
  parts:
  - **a downward directional "truss" light**, 6,000 lx: many lamps
    together act like even light from straight above;
  - **a 3 × 3 grid of point lamps**, 3.5 m up above the cage roof,
    600,000 lm each, each with an unlit emissive panel so it can be
    seen;
  - **global ambient fill** at 600 cd/m². From the driver's eye most of
    the robots is vertical faces, which overhead light only
    grazes.

  The first try was the lamps alone at 150,000 lm, about 1,000 lx under
  each, and it was "really dark" (user, 2026-10-10).

## Assertions

- The drawn floor and walls use the physics constants, never their
  own numbers.
- No light in the scene has `shadow_maps_enabled`.
