# Explicit motor torque on a light rotor rings unless the motor's reflected inertia is in it, and a lumped reflected inertia needs its chassis reaction corrected

Motor torque is computed from last step's rotor speed and applied as an
external torque, which is explicit Euler on the back-EMF damping
`b = ratio² · η / (kv² · R)`. It is stable only while the rotor's time
constant `J / b` is longer than about half the physics step.

What happened (2026-10-10):
- Regearing the fixture from 6:1 to 22.5:1 raised `b` 14×.
- A 68 g wheel (J ≈ 3e-5 kg m²) then had τ ≈ 0.1 ms against a 0.25 ms
  step.
- Each wheel's speed rang between the ±40 A current limits. Sampled, it
  looked like the two wheels fighting: one braking at full current while
  it overspun 30%, the other driving at full current while it lagged.
- At 6:1 it was hidden: straight-line speed still read 97.6%.

The fix is physical, not numerical:
- Add the motor's rotor inertia reflected through the gearbox,
  `J_motor · ratio²`, to the rotor body about its axis. At 22.5:1 it is
  80× the wheel's own.
- **But turn off rapier's gyroscopic term on wheel bodies.** The lumped
  inertia overstates their angular momentum about `ratio` times. In a
  pivot turn, the fake gyroscopic torque spun the rims up to 6× no-load.

- **And correct the reaction on the chassis.** The lump returns all of
  `J·r²·α` to the chassis; a real gearbox returns about `J·r·α`.
  Without the correction the fixture flipped end over end on every hard
  throttle reversal. `gear_coupling.rs` adds the difference, from the
  kinetic energy `½J(rω_w − (r−1)ω_c)²` against `½Jr²ω_w²`.

How to apply:
- When a rotor misbehaves at high ratio, compute `J / b` against `dt`
  before suspecting contacts or joints.
- Never set a motor's `rotor_inertia_kg_m2` to 0 for a geared rotor.
