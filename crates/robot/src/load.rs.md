# load.rs

## Function

The one entry point: `parts.json` bytes + `robot.toml` text + motor
library text → `RobotModel` (docs/drivesim.md §5).

## Interface

`load(parts_json, robot_toml, motors_toml) -> Result<RobotModel,
LoadError>`. `LoadError` names each mistake: `Parse`, `Frame`,
`UnknownMaterial`, `UnknownMotor`, `NoParts`, `TwoRoles`,
`TwoOverrides`, `NoAxis`, `NoChassis`, `MissingMu`, `BadSide`,
`BadChannel`, `BadMass`; its `Display` is written for the role
conversation to show a user.

## Implementation

1. Part masses: a mass override, else density × volume; then scaled to
   `total_mass_kg` if set (the CoM does not move, because every part
   scales by the same factor).
2. Each part's CoM and unit-density tensor are rotated into the robot
   frame and the tensor scaled by mass / volume.
3. Wheels then weapons claim parts by pattern; a part claimed twice is
   an error; the rest is the chassis.
4. Each rotor's axis comes from `axis_part` or the first claimed part
   with one; a wheel's side defaults to the sign of its axle centre's y.

## Assertions

- Tested on `fixtures/simple_bot` (crates/robot/tests/load_fixture.rs):
  analytic masses and box inertia, axes and sides, mass scaling keeping
  the CoM, overrides, a Y-up frame, and nine error cases.
