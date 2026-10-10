# ai_driver.rs

## Function

The AI opponent's driver (user, 2026-10-10: "basic ai driver opponent",
chase + unstick, weapon always at full). It turns toward the robot,
drives at it, and backs out when wedged, producing the same `Duties` a
radio through the mixer would.

## Interface

- `Pose { x, y, heading, speed, yaw_rate }`: a robot on the floor (m,
  rad CCW from +x, m/s along its heading, rad/s CCW).
- `AiDriver::default()`, `AiDriver::tick(&mut self, me, target, dt) ->
  Duties`: one control tick.
- `wrap(a)`: an angle into (−π, π].
- Tuning constants: `KP`, `KD`, `MAX_TURN`, `STUCK_THROTTLE`,
  `STUCK_SPEED_M_S`, `STUCK_S`, `REVERSE_S`, `REVERSE_LEFT`,
  `REVERSE_RIGHT`.

## Implementation

- **Pure.** No rapier: it sees two planar poses (`opponent::pose_of`
  makes them), so steering is unit-tested without a world.
- **Steering:** a PD controller on heading error (`KP` 2 stick/rad, `KD`
  0.15 stick per rad/s), capped at `MAX_TURN` 0.7: full stick spins the
  fixture at ~24 rad/s, far past what aiming needs. Throttle is
  `1 − |err| / 90°`, so it turns in place first and drives once lined up.
- **Arcade mixing as `mixer::mix` does it,** left = throttle − ccw,
  right = throttle + ccw, clamped; wheel direction (`forward_sign`) stays
  the drivetrain's job.
- **Stuck:** commanding ≥ 0.5 throttle while slower than 0.1 m/s for
  1 s → full reverse on the left, −0.4 on the right for 0.5 s, so the
  next approach comes from another angle. Pushing a robot that does move
  doesn't count as stuck.
- **Weapon always `Some(1.0)`** (user's choice). On the vertical-spinner
  fixture that means hits on walls throw it back and often flip it
  before the stuck timer fills (tests/ai.rs).
- **Upside down is `pose_of`'s job** (`opponent.rs`): it hands over the
  heading the wheels actually drive along, so the driver needs no
  inverted mode (user, 2026-10-10: "drive inverted").

## Assertions

- Duties are always within [−1, 1]; the weapon is always `Some(1.0)`.
- A target on the left gives `left < right`.
- Tested inline: turn sign; dead ahead is full throttle and weapon;
  target behind turns in place; 1 s wedged backs out for 0.5 s then
  chases; moving resets the stuck timer; `wrap`.
