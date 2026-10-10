# make_fixture.py

## Function

Builds `tests/fixtures/simple_bot.step`, the importer's known-answer
robot, laid out like a real 12 lb vertical spinner so it also drives
like one (user, 2026-10-10):
- an aluminium base plate and rails, a top plate, a rear bar, a battery
  and an ESC;
- two steel forks (a triangular prism, the wedge and front skids);
- two inline drive motors and two `Wheel` instances, 30 mm behind
  centre;
- a vertical `Weapon` disc between the forks.

The script's docstring holds the part table.

## Interface

`python tools/import/tests/make_fixture.py` rewrites the fixture. The
`.step` is checked in; tests read it and never run this script.

It replaced a 200 × 160 × 40 mm box with a horizontal disc. That robot
was too easy to spin, a compact brick with its CoM on the axle; it also
rocked between chassis edges and rested 5.7° tilted.

## Implementation

- Built with OCC's XCAF document so instance names and the shared
  product are real STEP structure (`NEXT_ASSEMBLY_USAGE_OCCURRENCE`), the
  same thing CAD exports produce.
- `TDataStd_Name.Set(label, str)`: pythonocc takes a plain `str` here,
  not an `ExtendedString`.
- **Products are shared** where a real CAD export would share them:
  `Rail`, `Fork`, `Wheel` and `DriveMotor` are each placed twice, so the
  importer's instance and product naming is exercised.
- Instance names (`wheel_L`, `wheel_R`, `weapon`) are kept from the old
  fixture; server tests look for them.

## Assertions

- The geometry in the docstring matches the numbers `test_import.py`,
  `load_fixture.rs`, `check_fixture.rs` and the server tests expect.
  Change them together, and re-run `make fixtures` to refresh
  `fixtures/simple_bot/`.
