# mass_props.py

## Function

Exact mass properties of one solid at unit density, so the robot file
can apply any material later without re-reading the STEP
(docs/drivesim.md §4–5).

## Interface

`mass_props(solid) -> dict`, all in the assembly frame:

| key | unit | meaning |
|---|---|---|
| `volume_m3` | m³ | B-rep volume |
| `com_m` | m | centre of volume |
| `inertia_m5` | kg m² at 1 kg/m³ | 3×3 tensor about `com_m`, global axes |

Multiply `volume_m3` and `inertia_m5` by a density in kg/m³ for mass (kg)
and inertia (kg m²).

## Implementation

- `BRepGProp.VolumeProperties` integrates the B-rep exactly; its
  `MatrixOfInertia` is already about the centre of mass, in global axes
  (checked against a disc's `m r²/2` in the tests).
- OCC reads STEP in mm, so lengths scale by 1e-3, volume by 1e-9 and the
  tensor by 1e-15.

## Assertions

- Tested: the fixture's box and cylinders give their analytic volumes,
  centres and disc inertia to 1e-9 relative.
- Never derived from the mesh: tessellation would lose volume on every
  curved face.
