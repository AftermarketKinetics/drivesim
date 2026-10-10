# shapes.rs

## Function

Collision shapes for a robot body, from its parts' meshes in the robot
frame (docs/drivesim.md §6).

## Interface

- `body_shapes(parts) -> Vec<SharedShape>`: convex hulls of the body's
  `MAX_HULLS_PER_BODY` (12) biggest parts by enclosed volume, each cut
  to at most `MAX_HULL_VERTS` (32) vertices. Degenerate meshes are
  skipped.
- `MAX_HULLS_PER_BODY` = 12.
- `MAX_HULL_VERTS` = 32.
- `mesh_volume(tris) -> f32`: enclosed volume of a closed, outward-wound
  mesh.

## Implementation

- **Why a cap, and no decomposition:** the first scheme (2026-10-09) gave
  every part a hull, or a V-HACD decomposition when the hull overfilled
  the part by 1.5×. The first real robot, Seraphs V1 (349 parts,
  2026-10-10), then had 695 colliders across it and its opponent copy,
  3,619 convex pieces among them. It took 46 s to build and ran at 0.01×
  real time; the user found it undrivable at 0.3× in the browser.
  Measured alternatives on that robot:
  - one hull per part, no decomposition: 691 colliders, 0.03×;
  - one hull per body: 13 colliders, 8.2×, but it fills gaps such as
    between the fixture's forks;
  - the 12 biggest parts' hulls (chosen): 35 colliders, 6.3× (3.6× with
    the weapon spinning and driving), and the outline stays made of real
    parts.
- **Why a vertex cap:** collision checks and the per-step bounding box
  update scale with hull vertex count. Obliterator 6 v72 (2026-10-10)
  had 23 hulls with up to 260 vertices, 2,336 in total. Capping each
  at 32 took its step from 70 to 54 µs natively.
- **How a hull is cut** (private `spread_subset`): keep the six
  axis-extreme vertices first, so the bounding box (and the lowest
  point, which sets ride height) stays exact. Then repeatedly add the
  vertex farthest from those kept, and rebuild the hull from the subset.
  Round parts come out as 32-vertex polytopes, slightly smaller than
  the true hull between the kept vertices.
- **What it costs:** small parts (fasteners, magnets, stator plates)
  never collide, though their mass still counts from the model. A
  concave big part becomes its hull.

## Assertions

- Tested: a box is its own hull, with its exact volume; of 20 parts only
  the 12 biggest get hulls; a ~260-vertex sphere is cut to at most
  `MAX_HULL_VERTS` vertices, keeping its bounding box.
- At most `MAX_HULLS_PER_BODY` shapes per body, each with at most
  `MAX_HULL_VERTS` vertices.
- A cut hull keeps its source hull's axis-aligned bounding box exactly.
