# wall_markings.rs

## Function

Distance markings on the cage walls, so the driver can judge distance
and speed from the box (user, 2026-10-10: "rulers and black/white squares
on the wall to denote meters").
- **Squares:** 1 m black and white squares in one row from the floor
  up, on the far wall and both side walls.
- **Ruler:** red ticks along the base of all four walls, every 0.1 m,
  longer at 0.5 m and longest at each metre.

## Interface

- `spawn_wall_markings(commands, root, meshes, mats)`, called by
  `arena_render::spawn_arena`.
- `SQUARE_M` = 1.0.

## Implementation

- **Counting:** each wall counts from its left-hand corner as seen from
  inside the cage, so every square's edge is a whole metre from a
  corner. The 4.877 m wall ends in a part square of 0.877 m.
- **The near wall gets the ruler only.** The driver looks through it,
  and squares there would block the view of the cage.
- **Drawing:** every marking is one shared unit cuboid, scaled, sitting
  2 mm (squares) or 4 mm (ticks) in front of the wall's inner face so
  they never z-fight it or each other.
- **Ticks are red,** so they show on both the black and the white
  squares.
- **The panel frame** is (along, inward, along × inward). It must be
  right-handed to become a rotation; along × inward comes out −z for
  every wall, which is harmless because the panel is symmetric.

## Assertions

- Markings are measured from the physics constant `ARENA_SIDE_M`, so
  they match the simulated cage.
- No squares on the near (−x) wall.
