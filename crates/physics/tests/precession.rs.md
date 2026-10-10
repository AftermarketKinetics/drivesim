# precession.rs

## Function

The weapon's gyroscopic behaviour, the validation docs/drivesim.md §9
asks for, with gravity off and no contacts.

## Interface

Tests:
- `torque_across_the_spin_precesses_at_tau_over_l`: 0.5 N m about x,
  across the spin, on the chassis, with the weapon at 1,000 rad/s about
  its own axis. After 1 s the spin axis has swung toward x by τ/L·t,
  within 10%, and tipped about x by less than 20% of that.
- `without_spin_the_same_torque_just_tips_it`: the contrast case. With
  the weapon still, 0.5 N m for 0.1 s rolls the robot by τt²/(2 I_xx),
  within 15%, about the combined inertia. It used to check a fixed
  0.05 rad, which the fixture stopped reaching when it got twice as wide
  (2026-10-10).
- `nothing_driven_conserves_angular_momentum`: the chassis tumbling and
  the weapon spinning; total angular momentum about the system CoM
  stays within 1% over 1 s.

## Implementation

The tests work from the weapon's own axis, never a fixed one. The
fixture's disc turned from a z axis to a y axis when it became a
vertical spinner (2026-10-10). L uses the weapon body's inertia as
spawned, including its motor's reflected inertia (spawn.rs).

Angular momentum is summed from each body's principal inertia, in its
world-rotated principal frame, plus `m · r × v` about the system CoM.
Measured after one step, because rapier fills in world mass properties
on the first step (before that they read NaN).

## Assertions

- Precession comes from rapier's own gyroscopic term and the joint. Nothing
  in the sim applies it by hand.
