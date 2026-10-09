# rotation_axis.py

## Function

Reports the axis a solid turns about, if it has one, plus the evidence
the role conversation uses to tell a wheel or weapon from a plate with a
hole (docs/drivesim.md §4). Reports only; it never decides a role.

## Interface

- `rotation_axis(solid) -> dict | None`: `None` when the solid has no
  revolved faces. Otherwise:
  - `point_m`, `dir`: the axis line; `dir` is a unit vector whose
    largest component is positive.
  - `curved_fraction`: share of the surface that is revolved about this
    axis.
  - `cap_fraction`: share that is planar and square to the axis.
- `radial_extent(axis, points_m) -> dict`: `radius_m` (largest distance
  from the axis), `width_m` (span along it) and `centre_m` (mid-span on
  the axis), from mesh points.
- `distance_to_axis(axis, point_m) -> float`: m from a point to the line;
  `step_import` stores it for the part's centre as `com_offset_m`.

## Implementation

- Revolved faces are cylinders, cones, tori and surfaces of revolution.
  They are grouped by line (same direction to 1e-6, same position to
  0.01 mm) and the group with the most area wins.
- The curved and cap shares are reported separately. A single combined
  score scored Momentum's flat floor plate 89% "revolved", because its
  large faces are square to a bolt hole's axis (2026-10-09). On
  Momentum's parts the separating number is the centre's distance from
  the axis: wheel and blade 0.0 mm, floor 126 mm, fork tine 68 mm.
- Radius and width come from the mesh, not the B-rep, so blade teeth and
  lugs count toward the outer radius.

## Assertions

- A plain box returns `None`.
- A cylinder or disc returns its own axis, `curved + cap = 1`, a centre
  offset of 0, and radius and width within 1 µm (fixture tests).
