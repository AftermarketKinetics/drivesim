# ai.rs

## Function

The AI opponent in the arena with the fixture robot: the countdown, the
chase, and walls.

## Interface

Tests:
- `it_waits_out_the_countdown`: no movement (< 1 cm) until just before
  `COUNTDOWN_S`; `countdown_left()` is `None` after it.
- `the_copy_and_the_box_have_no_countdown`.
- `it_reaches_an_idle_robot`: from the start spots, chassis centres come
  within 0.4 m in 3 s (measured 0.34 m: touching).
- `it_does_not_stay_pinned_on_a_wall`: driving at a robot parked outside
  the −x wall, it reaches within 0.4 m of the wall and later backs off
  it more than 0.1 m.
- `a_rival_drives_with_its_own_model`: `Sim::versus` with a rival (the
  fixture renamed "rival" and scaled to 9 kg) keeps the rival's model
  and mass on the opponent and the fixture's on the robot, and the
  heavier rival still comes within 0.45 m of the robot in 4 s.

## Implementation

Tests skip the countdown by setting `time_s = COUNTDOWN_S`. The wall
test puts the target outside the arena, so the AI keeps driving into
the wall. With the disc at full the stuck timer never fills: each hit
on the wall throws it back, often flipping it (measured 2026-10-10:
1.6 m back within 4 s, up.z down to −0.93). That is why the test checks
"not pinned" rather than "reversed"; stuck recovery is tested in
`ai_driver.rs`.

## Assertions

- Robots are moved by shifting every body of theirs together, never the
  chassis alone.
