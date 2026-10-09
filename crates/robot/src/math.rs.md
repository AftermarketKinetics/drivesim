# math.rs

## Function

The vector and 3×3 operations the mass rollup needs, on plain `f64`
arrays, so the crate has no math dependency.

## Interface

`V3`, `M3`, `ZERO`; `add`, `sub`, `scale`, `dot`, `cross`, `mat_vec`,
`mat_add`, `mat_scale`, `rotate_tensor` (`r·m·rᵀ`), `parallel_axis`
(`m(|d|²E − d dᵀ)`).

## Implementation

f64 because a robot sums parts whose masses differ by 10⁴ (a screw and a
blade). The physics crate converts to rapier's f32 types at the boundary.

## Assertions

- Pure functions; exercised through `mass` and `frame` tests.
