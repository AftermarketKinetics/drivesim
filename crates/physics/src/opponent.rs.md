# opponent.rs

## Function

The opponent robot (docs/drivesim.md §6): the driven robot's copy, or a
rival robot picked on the sim page (user, 2026-10-10). Undriven (braked,
weapon off), or driven by `ai_driver` once a 3 s countdown ends (user,
2026-10-10). `Sim` owns at most one.

## Interface

- `OPPONENT_DUTIES`: armed, sticks centred: wheels brake, weapon off.
- `COUNTDOWN_S` 3.0: sim seconds before the AI starts.
- `Opponent { robot, model, ai, duties }` (plus its `Drivetrain`,
  crate-only):
  - `Opponent::new(robot, model, ai: bool)`; `model` is the opponent's
    own (a clone of the driven robot's for the copy);
  - `control(&mut self, world, target, time_s, dt)`: sets `duties` for
    this control tick.
- `pose_of(body) -> ai_driver::Pose`.

## Implementation

- Moved out of `sim.rs` with the AI (2026-10-10), which would otherwise
  have pushed it past ~200 lines.
- **The countdown is sim time,** not wall time, so it is deterministic
  in tests and pauses with the sim. Only the AI waits: the driven robot
  may move during it, to line up.
- Before the countdown ends, the AI copy holds `OPPONENT_DUTIES`, so it
  sits exactly like the undriven copy.
- **`pose_of` drives inverted** (user, 2026-10-10): upside down
  (chassis up.z < 0) the heading and speed come from the chassis −x
  axis instead of +x. The floor then meets each wheel on the other side
  of the hub, so forward spin rolls the robot backwards, while its left
  wheels stay on the left of that travel direction; so `ai_driver`
  steers correctly with no change of its own.
- Not tested in the arena: the fixture rolled over rests on its disc,
  not its wheels, and with the disc at full it cartwheels instead of
  driving (measured 2026-10-10: weapon stalled at ~10 rad/s, chassis up
  to 0.74 m). Unit-tested on bare bodies instead.

- **The opponent owns its model** (2026-10-10, for rivals): `Sim`
  drives its drivetrain with `Opponent::model`, never the driven
  robot's.

## Assertions

- An opponent with `ai: None` always gets `OPPONENT_DUTIES`.
- Duties change only in `control`, once per control tick.
- Tested inline: upright heads along +x; rolled over (π about x) heads
  along −x with speed negated.
