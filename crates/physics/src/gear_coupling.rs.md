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
- `MAX_SELF_FRACTION` 0.5 and `coupling_scale(rotors, chassis) -> f32`:
  - `rotors` are (unit axis, motor J, ratio) in the chassis frame;
  - `chassis` is the chassis inertia about its CoM;
  - the result is the factor (≤ 1) every coupling torque of the robot
    is multiplied by, so that the rotors' summed chassis self-term
    `(r−1)²J` (projected on each rotor's axis) stays within
    `MAX_SELF_FRACTION` of the chassis inertia about that axis.
  - `drivetrain.rs` works it out once per robot.

## Implementation

The difference between the real and the lumped kinetic energy:

- `KE_real = ½ J (r ω_w − (r−1) ω_c)²`
- `KE_lump = ½ J r² ω_w²`

As torques, it gives:
- **on the rotor:** `r(r−1) J α_c`;
- **on the chassis:** `r(r−1) J α_w − (r−1)² J α_c`.

**Why the lagged accelerations are stable, and the cap:** the chassis
term's self-coupling, `(r−1)² J`, is about 2.3e-3 kg m² at 22.5:1,
against the fixture chassis's ~0.02 pitch inertia. An explicit added
inertia is stable while it is well under the body's own. A light robot
on high-ratio gearmotors breaks that. A 0.96 kg beetle on 27:1 Repeat
Max drive sat at 1:1 (2.2e-3 against 2.2e-3) and blew itself across the
arena at 65 m/s with every motor unpowered; its unpowered weapon spun to
28,000 rpm (2026-10-11, reported live). `coupling_scale` caps the term
at `MAX_SELF_FRACTION` of the chassis inertia, scaling the whole
correction down for such robots (user's choice of fix).
- **Measured:** capped at 0.75 or 0.5 the beetle sat still (0.1 m/s);
  0.25 also held it, but left the 1 lb house robot tilting 11° on
  weapon spin-up instead of 4.6°.
- The 12 lb fixture and the 3 and 30 lb house robots stay at scale 1.
- **The ratio is necessary, not sufficient:** the fixture with its
  chassis inertia cut 100× didn't diverge, while the beetle (with
  wheels whose mass sat 10 cm off their axles) did.
- **What a capped robot gives up:** part of the gearbox correction, so
  its wheels' spin-up reaction on the chassis is overstated, toward the
  flip-on-reversal the correction fixed.

## Assertions
- `coupling_scale` is in (0, 1]; 1 when no rotor has a gearbox (r = 1)
  or the chassis is heavy enough.
- Tested inline: a heavy chassis keeps scale 1; a light one is scaled to
  exactly the cap; a rotor across the wheels' axis adds nothing to it.
  In tests/drive.rs, a light-chassis fixture variant that reached
  150 m/s uncapped sits still.

- Tested (unit):
  - rigid rotation (`α_w = α_c`) leaves a net of `J α`, so the robot
    carries the motor rotor once, not `r²` times;
  - spin-up against a still chassis reacts `J r α`;
  - `r = 1` needs no correction.
- Tested (tests/drive.rs, `slamming_throttle_does_not_flip_it`): full
  forward, full reverse, full forward; the fixture stays upright
  (up.z > 0.9).
