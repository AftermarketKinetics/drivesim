# robot_file.rs

## Function

The `robot.toml` schema: everything a STEP cannot say
(docs/drivesim.md §5).

## Interface

`parse_robot_file(text) -> Result<RobotFile, toml::de::Error>`.

- `[robot]`: `name`, `up`, `forward`, `battery_v`, optional
  `total_mass_kg`.
- `[materials]`: `default`, then `[[materials.rule]]` with `parts` and
  either `material` or `density_kg_m3`; first match wins.
- `[mass_overrides]`: pattern → kg.
- `[[wheel]]` / `[[weapon]]`: `name`, `parts`, optional `axis_part`,
  `motor`, optional `[motor_override]`, `ratio`, `efficiency` (default
  1), wheels also `mu` and optional `side`.
- `[control]`: `kind = "mixer"` with 1-based `throttle`, `turn`,
  optional `weapon` and `arm`, `reverse_turn`; or
  `kind = "momentum-core"`.

## Implementation

Every table denies unknown keys: a typo such as `mu_typo` is a load
error, not a setting quietly ignored.

## Assertions

- Channel numbers are 1-based, as the radio shows them; `load` checks
  1..=16.
