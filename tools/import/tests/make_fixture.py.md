# make_fixture.py

## Function

Builds `tests/fixtures/simple_bot.step`, the importer's known-answer
robot, laid out like a real 12 lb vertical spinner so it also drives
like one (user, 2026-10-10):
- an aluminium base plate and rails, a top plate, a rear bar, a battery
  and an ESC;
- two steel forks (a triangular prism, the wedge and front skids);
- two inline drive motors and two `Wheel` instances, 30 mm behind
  centre, on a 440 mm track (the chassis is 400 mm wide since
  2026-10-10, user: "twice as wide"; the base plate went to 3 mm and the
  rear bar to 6 mm to stay inside 12 lb);
- a vertical `Weapon` disc between the forks, with one big tooth
  (user, 2026-10-10) and a balance hole.

The script's docstring holds the part table.

## Interface

`python tools/import/tests/make_fixture.py` rewrites the fixture. The
`.step` is checked in; tests read it and never run this script.

`make_fixture.py --scale S --name NAME OUT.step` writes the same robot
scaled by S about the origin, with assembly name NAME. That is how the
house robot's weight-class versions were made (user, 2026-10-11):
`simple_bot_1lb.step` ×0.43, `simple_bot_3lb.step` ×0.628 and
`simple_bot_30lb.step` ×1.375, all beside `simple_bot.step`. `make
fixtures` imports them into `fixtures/simple_bot_*`.

It replaced a 200 × 160 × 40 mm box with a horizontal disc. That robot
was too easy to spin, a compact brick with its CoM on the axle; it also
rocked between chassis edges and rested 5.7° tilted.

## Implementation

- **Scaling is real geometry** (`BRepBuilderAPI_Transform` on each
  product, instance offsets ×S), not a scaled placement, which STEP
  readers handle unevenly. Part and product names are unchanged, so a
  scaled robot's robot.toml rules match the same parts. The scale factors
  are chosen so each version, with its class's battery, ESC and motor
  masses, comes to ~97% of its class limit, as the 12 lb original does.

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

- **The weapon is balanced by construction.** `weapon()` fuses the
  tooth onto the disc, measures the first moment it adds with OCC, and
  cuts a through hole at r 40 mm *on the tooth's side* sized to remove
  exactly that moment (39.5 mm across). A hole opposite the tooth would
  double the imbalance. The first try did, and the importer reported the
  CoM 6.6 mm off the axis.
- The tooth sweeps r 91.2 mm (its tip corners), so the disc centre is
  raised to z = 95 mm (3.8 mm floor clearance), and the battery is
  shortened to x ≤ 70 to stay clear of the sweep.

## Assertions

- The geometry in the docstring matches the numbers `test_import.py`,
  `load_fixture.rs`, `check_fixture.rs` and the server tests expect.
  Change them together, and re-run `make fixtures` to refresh
  `fixtures/simple_bot/`.
- The weapon's CoM is on its axis (importer `com_offset_m` ≈ 0).
