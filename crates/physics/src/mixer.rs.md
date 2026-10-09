# mixer.rs

## Function

The generic control (`kind = "mixer"` in robot.toml): arcade mixing of
a CRSF frame into ESC duties, for any robot without its own firmware.

## Interface

- `Duties { left, right, weapon }`, each `Option<f32>`; `None` = no
  pulses.
- `MixerConfig { throttle, turn, weapon, arm, reverse_turn }`, with
  1-based channel numbers.
- `mix(frame: Option<&Frame>, cfg) -> Duties`.

## Implementation

- No frame (signal loss) or arm channel at or below 50% gives no pulses
  on every ESC, as a receiver failsafe set to "no pulses" would.
- left = throttle + turn, right = throttle − turn, clamped: turn stick
  right speeds the left side up, turning clockwise seen from above.
- The weapon channel is a throttle: −1 → 0, +1 → 1.

## Assertions

- Tested: full forward; turn right; disarmed and no signal; weapon
  throttle mapping.
