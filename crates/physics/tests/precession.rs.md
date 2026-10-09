# precession.rs

## Function

The weapon's gyroscopic behaviour, the validation docs/drivesim.md §9
asks for, with gravity off and no contacts.

## Interface

Tests:
- `torque_across_the_spin_precesses_at_tau_over_l`: 0.5 N m about x, on
  the chassis, with the weapon at 1,000 rad/s. After 1 s the spin axis
  has swung about y by τ/L·t, within 10%, and tipped about x by less
  than 20% of that.
- `without_spin_the_same_torque_just_tips_it`: the contrast case.
- `nothing_driven_conserves_angular_momentum`: the chassis tumbling and
  the weapon spinning; total angular momentum about the system CoM
  stays within 1% over 1 s.

## Implementation

Angular momentum is summed from each body's principal inertia, in its
world-rotated principal frame, plus `m · r × v` about the system CoM.
Measured after one step, because rapier fills in world mass properties
on the first step (before that they read NaN).

## Assertions

- Precession comes from rapier's own gyroscopic term and the joint. Nothing
  in the sim applies it by hand.
