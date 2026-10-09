# report.rs

## Function

Turns a robot bundle into the JSON report the role conversation reads
(docs/drivesim.md §4).

## Interface

`report(parts_json, robot_toml, motors_toml, mesh) -> serde_json::Value`;
`SETTLE_S` = 2.0.

- **On failure:** `{"ok": false, "error": "<LoadError or SpawnError
  text>"}`.
- **On success:** `{"ok": true, ...}` with these fields:
  - `name`, `total_mass_kg`, `com_m`, `inertia_kg_m2`;
  - `bodies[]`: `name`, `parts`, `mass_kg`;
  - `wheels[]` and `weapons[]`: `name`, `axis_dir`, `axis_point_m`,
    `radius_mm`, `no_load_rpm`, `no_load_tip_speed_m_s`, `side`, `mu`,
    `motor_kv`, `provisional_motor_fields`;
  - `rest`: `tilt_deg`, `com_height_mm`, `touching_floor`, `settled`.

## Implementation

Values are rounded for a reader (mm, 0.1°, 1 g), in f64, so the JSON
shows `34.8` rather than f32 noise. No-load speeds come from the motor
model at full duty and battery voltage.

## Assertions

- Tested (tests/check_fixture.rs): the fixture reports 4.54 kg, 2 wheels,
  the `disc` weapon, and a settled rest without the disc touching the
  floor. A bad `robot.toml` gives exit 1 with a JSON error.
