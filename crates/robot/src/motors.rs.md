# motors.rs

## Function

The motor library (`data/motors.toml`) and per-robot overrides
(docs/drivesim.md §5).

## Interface

- `MotorSpec { kv_rpm_per_v, r_ohm, i_max_a, pole_pairs (default 7),
  rotor_inertia_kg_m2 (default 0), source, provisional }`;
  `with(&MotorOverride)`, `kv_rad()`.
- `MotorOverride`: every number optional.
- `Library` (name → spec); `parse_library(text)`.

## Implementation

Units are as datasheets give them (kv in rpm/V); `kv_rad` converts for
the motor model. `source` and `provisional` travel with the spec, so a
placeholder is visible wherever the motor is used.

## Assertions

- Unknown keys are errors.
- Tested: defaults, kv conversion, overrides touching only named fields.
