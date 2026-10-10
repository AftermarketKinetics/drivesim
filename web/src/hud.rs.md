# hud.rs

## Function

The heads-up text: what a driver practising needs but can't judge from
the box. That is radio link, arming, speed, weapon speed, tilt and
physics headroom.

## Interface

- `HudText`: a component marking the text node. `main.rs` writes load
  errors into it.
- `spawn_hud(commands)`: the top-left text, "loading robot…" at first.
- `update_hud`: a system that, once the sim exists, shows:
  - the robot's name;
  - "radio connected" or "NO RADIO (move a stick…)";
  - "ARMED" or "disarmed";
  - ground speed (m/s), the first weapon's speed (rpm) and chassis tilt
    (°);
  - physics headroom (× real time).

## Implementation

- "ARMED" is read from the sim's own duties (some wheel or weapon duty is
  set), not from the radio's arm channel. It shows what the robot is
  actually doing.
- Tilt is the angle between the chassis's up axis and world up.
- Until the sim exists the text is left alone, so a load error written by
  `main.rs` stays on screen.

## Assertions

- The HUD only reads state; it never changes the sim or the radio.
