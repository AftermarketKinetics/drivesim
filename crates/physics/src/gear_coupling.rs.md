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
- `drivetrain.rs` adds these to the motor torques every physics step,
  using the previous steps' accelerations, smoothed.
- `self_ratio(rotors, chassis) -> f32` (g):
  - `rotors` are (unit axis, motor J, ratio) in the chassis frame;
  - `chassis` is the chassis inertia about its CoM;
  - the result is the rotors' summed chassis self-term `(r−1)²J`,
    projected on each rotor's axis, against the chassis inertia about
    that axis, taking the largest over the axes. 0 with no gearbox.
- `smoothing(g) -> f32`: `1/(1+g)`, the β of the filter below.
- `smoothed(prev, measured, beta) -> f32`: `prev + β (measured − prev)`.
- `drivetrain.rs` works out β once per robot and smooths every
  acceleration it feeds `gear_coupling`.

## Implementation

The difference between the real and the lumped kinetic energy:

- `KE_real = ½ J (r ω_w − (r−1) ω_c)²`
- `KE_lump = ½ J r² ω_w²`

As torques, it gives:
- **on the rotor:** `r(r−1) J α_c`;
- **on the chassis:** `r(r−1) J α_w − (r−1)² J α_c`.

**Why the accelerations are smoothed:** the chassis term's
self-coupling `K = (r−1)² J` is an added inertia, fed back a step late.
Unfiltered, `α = (τ − K α_prev)/I` diverges once `g = K/I` nears 1. The
12 lb fixture is at 0.12, but a 0.96 kg beetle on 27:1 Repeat Max drive
is at 1.8. It blew itself across the arena at 65 m/s with every motor
unpowered, and its unpowered weapon spun to 28,000 rpm (2026-10-11,
reported live).
- **The first fix, a cap, was wrong.** It scaled the whole correction
  to keep g at 0.5. That also scaled the cross term that cancels the
  lump's fake chassis reaction (`J r (r−1) α_w`, 26× the real one at
  27:1), so 73% of the fake reaction stayed. The beetle backflipped at
  half throttle with its weapon off, and in AI vs AI both robots flipped
  on launch (2026-10-11).
- **The filter:** a one-pole low-pass on the accelerations,
  `α_f ← α_f + β (α − α_f)`. The loop then gives
  `α_f ← (1 − β(1+g)) α_f + β τ/I`, which is stable while `β(1+g) < 2`.
  At `β = 1/(1+g)` it lands on the implicit answer `τ/(I+K)` in one step,
  for any g (user's choice of fix, filtering every robot).
- **Measured:** uncapped and unfiltered, the beetle still blows up.
  Filtered at 0.15–0.5, it sits still unpowered and stays upright at full
  throttle (≤ 20° pitch).
- **What a heavy robot pays:** at g = 0.12, β = 0.89, so it is barely
  changed. Still, the 12 lb fixture's weapon-off ram went from 1.6 to
  2.26 m/s in its hardest step.

## Assertions
- `smoothing` is in (0, 1]; 1 exactly when no rotor has a gearbox (r = 1).
- Tested inline: the filtered self-term lands on `τ/(I+K)` at once for
  g from 0.1 to 10; `self_ratio` sums two 27:1 wheels on their axis; a
  rotor across the wheels' axis adds nothing to it; no gearbox is not
  filtered. In tests/drive.rs, the light-chassis fixture variant (which
  reached 150 m/s unfiltered) sits still unpowered and stays upright
  through full throttle, full reverse and full throttle.
- Tested (unit):
  - rigid rotation (`α_w = α_c`) leaves a net of `J α`, so the robot
    carries the motor rotor once, not `r²` times;
  - spin-up against a still chassis reacts `J r α`;
  - `r = 1` needs no correction.
- Tested (tests/drive.rs, `slamming_throttle_does_not_flip_it`): full
  forward, full reverse, full forward; the fixture stays upright
  (up.z > 0.9).
