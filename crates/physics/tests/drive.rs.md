# drive.rs

## Function

The fixture robot in the arena, driven through the mixer as the radio
would drive it.

## Interface

Tests:
- `throttle_drives_straight_near_no_load_speed`: 40% throttle for 1 s:
  speed within 5% of no-load, heading within 0.1 rad.
- `turn_right_turns_clockwise`.
- `full_turn_is_bounded_by_the_wheels_and_slowed_by_the_forks`: full
  turn for 2 s: yaw rate between 50% and 102% of 2 v / track.
- `wheels_roll_without_ringing`: 40% for 0.8 s: each rim within 5% of
  ground speed, none past no-load.
- `slamming_throttle_does_not_flip_it`: full, full reverse, full, 0.5 s
  each: up.z stays above 0.9.
- `the_box_weighs_what_the_robot_does`.
- `disarmed_does_not_move`.
- `weapon_spins_up_to_no_load_and_the_robot_stays_upright`: within 5%
  after 3 s.
- `driving_into_the_box_pushes_it`.
- `the_fixture_rests_on_its_wheels_and_fork_tips`: `rest::settle`:
  under 1° nose-down, on both wheels and the chassis (the fork tips),
  with the disc clear.

## Implementation

**The straight-line run moves the box out of the way** (`sim_alone`).
With VytaFlex 40 tyres (mu 1.4, 2026-10-10) the fixture reached the box
inside the 1 s run. Earlier history: At 40% it
reaches speed by about 0.5 s and hits the box at about 1.2 s. A 1.5 s
run first failed at "62% of no-load" only because the box was in the
way (2026-10-09). Since the fixture was regeared (2026-10-10) the run
is at 40% (1.8 m/s) instead of 10%.

**Turn rate is under the wheel limit by design.** The vertical-spinner
fixture (2026-10-10) carries about 25% of its weight on steel fork tips,
which drag sideways in a pivot, so full stick settles at about 24 of
the ideal 37 rad/s with the wheels slipping. The test bounds it: never
past 2 v / track (wheels outrunning their motors), and not under half
(something stuck).

**Ringing.** Without reflected motor inertia the regeared fixture's
wheels rang between their current limits, one braking at full current
while it overspun, and straight runs came out at 92% of no-load. That is
why `wheels_roll_without_ringing` exists. Rapier's soft contacts creep
about 3.5% at steady rolling, hence its 5% bound.

## Assertions

- Expected speeds come from `motor::no_load_speed`, never from a
  previous run.
