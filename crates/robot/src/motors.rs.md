# motors.rs

## Function

The motor library (`data/motors.toml`) and per-robot overrides
(docs/drivesim.md §5).

## Interface

- `MotorKind { Brushless (default), Brushed }`, lowercase in TOML.
- `MotorSpec { kind, kv_rpm_per_v, r_ohm, i_max_a, pole_pairs (default 7),
  rotor_inertia_kg_m2 (default 0), source, provisional }`;
  `with(&MotorOverride)`, `kv_rad()`.
- `MotorOverride`: every number optional.
- `Library` (name → spec); `parse_library(text)`.

## Implementation

Units are as datasheets give them (kv in rpm/V); `kv_rad` converts for
the motor model. `source` and `provisional` travel with the spec, so a
placeholder is visible wherever the motor is used. `kind` is not
overridable and does not change the physics: a brushed motor is the
same DC model (kv, terminal resistance, current limit), and pole_pairs
is ignored for it.

## Assertions

- Unknown keys are errors.
- `kind` defaults to brushless, so older libraries still parse.
- Tested: defaults, kind parsing, kv conversion, overrides touching only named fields.
