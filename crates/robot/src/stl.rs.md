# stl.rs

## Function

Reads the importer's binary STL meshes (metres, wound outward) into
triangles for colliders and drawing.

## Interface

`type Tri = [[f32; 3]; 3]`; `parse_stl(bytes) -> Option<Vec<Tri>>`
(`None` if truncated or too short for a header).

## Implementation

Reads the count, checks the length covers `84 + 50·n` bytes, and skips
each facet's stored normal: the winding is the truth.

## Assertions

- Tested: one triangle read exactly; truncated input is `None`.
