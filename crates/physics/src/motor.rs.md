# motor.rs

## Function

The motor plus ESC as a DC motor, carried over from Momentum's 2D sim
(2026-10-09): what turns a duty into rotor torque. Brushless and brushed
motors (`MotorKind`) share it; `kind` and `pole_pairs` are not read.

## Interface

- `rotor_torque(duty: Option<f32>, rotor_w, rotor: &Rotor, battery_v) ->
  f32`: torque on the rotor about its axis, N m. `rotor_w` is the
  rotor's speed relative to the chassis, in the duty's sign convention.
  `None` freewheels (0).
- `no_load_speed(duty, rotor, battery_v) -> f32`: where the torque
  reaches zero, rad/s.

## Implementation

`I = clamp((d·V − ω·ratio/kv)/R, ±i_max)`, torque `I/kv · ratio · η`,
with kv in rad/s per volt. A duty below the back-EMF gives negative
current: complementary PWM brakes, it does not coast. No battery sag, no
inductance, no ESC ramp.

## Assertions

- Tested: current-limited stall torque; zero torque at no-load speed; a
  low duty brakes a fast rotor; `None` gives zero.
