# parts.rs

## Function

The Rust side of the importer's `parts.json` (format 1,
`tools/import/step_import.py.md`).

## Interface

- `FORMAT` = 1; `parse_parts(bytes) -> Result<PartsDoc, PartsError>`
  (fails on a different format).
- `PartsDoc { format, source, parts }`; `Part { id, instance, product,
  path, color, mesh, bbox_m, volume_m3, com_m, inertia_m5,
  rotation_axis }`; `Axis { point_m, dir, curved_fraction, cap_fraction,
  radius_m, width_m, centre_m, com_offset_m }`.
- `Part::matches(pattern)`: true if the pattern matches the instance,
  the product or the path.

## Implementation

Field names mirror the JSON keys exactly so serde needs no renames; a
key renamed in the importer must bump its `FORMAT` and this one together.

## Assertions

- `FORMAT` here equals `FORMAT` in `step_import.py`.
