# make_fixture.py

## Function

Builds `tests/fixtures/simple_bot.step`, the importer's known-answer
robot: a box chassis, one `Wheel` product placed twice (`wheel_L`,
`wheel_R`) touching the floor, and a `Weapon` disc above.

## Interface

`python tools/import/tests/make_fixture.py` rewrites the fixture. The
`.step` is checked in; tests read it and never run this script.

## Implementation

- Built with OCC's XCAF document so instance names and the shared
  product are real STEP structure (`NEXT_ASSEMBLY_USAGE_OCCURRENCE`), the
  same thing CAD exports produce.
- `TDataStd_Name.Set(label, str)`: pythonocc takes a plain `str` here,
  not an `ExtendedString`.

## Assertions

- The geometry in the header comment matches the numbers
  `test_import.py` expects; change both together.
