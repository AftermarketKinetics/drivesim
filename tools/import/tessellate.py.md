# tessellate.py

## Function

Turns a solid into triangles in metres and writes them as binary STL:
the meshes the web app draws and the physics decomposes into convex
colliders (docs/drivesim.md §4, §6).

## Interface

- `triangles(solid, linear_mm=0.1, angular_rad=0.3) -> list`: triangles
  as three `(x, y, z)` points in metres, wound outward.
- `write_stl(path, tris)`: binary STL, with normals computed from the
  winding.

## Implementation

- `BRepMesh_IncrementalMesh` at 0.1 mm chord error and 0.3 rad: a 43 mm
  wheel's mesh radius stays within 0.1 mm of the true radius, and a whole
  part meshes in well under a second.
- Faces OCC marks `REVERSED` get their winding swapped, so every normal
  points out of the solid. Collision decomposition and back-face culling
  both depend on it.
- STL has no unit field; the 80-byte header says "metres".

## Assertions

- Tested: every fixture mesh has a positive signed volume within 1% of
  the B-rep volume, which only holds when every triangle is wound
  outward and the mesh is closed.
