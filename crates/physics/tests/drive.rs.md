# drive.rs

## Function

The fixture robot in the arena, driven through the mixer as the radio
would drive it.

## Interface

Tests:
- `throttle_drives_straight_near_no_load_speed`: 10% throttle for 1 s:
  speed within 5% of no-load (measured 97.6%), heading within 0.1 rad.
- `turn_right_turns_clockwise`.
- `disarmed_does_not_move`.
- `weapon_spins_up_to_no_load_and_the_robot_stays_upright`: within 5%
  after 3 s.
- `driving_into_the_box_pushes_it`.
- `the_fixture_rests_on_its_wheels_and_a_chassis_edge`: `rest::settle`.

## Implementation

**The straight-line run stops at 1 s, before the box.** It reaches speed
by about 0.6 s and hits the box at about 1.3 s. A 1.5 s run first failed
at "62% of no-load" only because the box was in the way (2026-10-09).

## Assertions

- Expected speeds come from `motor::no_load_speed`, never from a
  previous run.
