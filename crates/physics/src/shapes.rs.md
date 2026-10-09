# shapes.rs

## Function

Collision shapes from a part's mesh, in the robot frame.

## Interface

- `part_shape(tris) -> Option<SharedShape>`: the convex hull, or a
  V-HACD decomposition when the hull's volume exceeds `HULL_SLACK` (1.5)
  times the mesh's.
- `mesh_volume(tris) -> f32`: enclosed volume of a closed, outward-wound
  mesh.

## Implementation

- **The hull comes first:** the design said V-HACD per body, but most
  combat-robot parts (plates, blocks, bars) are close to convex, so a
  hull per part is exact enough and far cheaper. Only parts the hull
  overfills, such as brackets and rings, pay for decomposition.
  Decided 2026-10-09; this is a proposal pending the user's review.
- STL triangles share no vertices; V-HACD needs an indexed mesh, so
  vertices within 1 µm are merged.

## Assertions

- Tested: a box is its own hull with its exact volume; an L-shape is
  decomposed into a compound.
