# gear_coupling.rs

## Function

The correction that turns a lumped reflected inertia into a real
gearmotor (docs/drivesim.md §6).

`spawn.rs` adds the motor rotor's inertia `J·r²` to each rotor body so
that motor torque spins the wheel up as slowly as it really does. On its
own, though, that lump also returns the whole `J·r²·α` to the chassis as
reaction. A real gearbox returns only about `J·r·α`, because the motor's
rotor turns at `r·ω_rotor − (r−1)·ω_chassis`, and the rest is the gear
housing's reaction. Without this correction, a hard throttle reversal
kicked the fixture's chassis about 22× too hard, and it flipped end over
end (2026-10-10).

## Interface

- `gear_coupling(j_motor, ratio, alpha_rotor, alpha_chassis) -> (on_rotor,
  on_chassis)`:
  - torques in N m about the rotor's axis;
  - accelerations are the two bodies' absolute angular accelerations
    about that axis, in rad/s².
- `sim.rs` adds these to the motor torques every physics step, using the
  previous step's accelerations.

## Implementation

The difference between the real and the lumped kinetic energy:

- `KE_real = ½ J (r ω_w − (r−1) ω_c)²`
- `KE_lump = ½ J r² ω_w²`

As torques, it gives:
- **on the rotor:** `r(r−1) J α_c`;
- **on the chassis:** `r(r−1) J α_w − (r−1)² J α_c`.

**Why the lagged accelerations are stable:** the chassis term's
self-coupling, `(r−1)² J`, is about 2.3e-3 kg m² at 22.5:1, against
the fixture chassis's ~0.02 pitch inertia. An explicit added inertia is
stable while it is well under the body's own.

## Assertions

- Tested (unit):
  - rigid rotation (`α_w = α_c`) leaves a net of `J α`, so the robot
    carries the motor rotor once, not `r²` times;
  - spin-up against a still chassis reacts `J r α`;
  - `r = 1` needs no correction.
- Tested (tests/drive.rs, `slamming_throttle_does_not_flip_it`): full
  forward, full reverse, full forward; the fixture stays upright
  (up.z > 0.9).
