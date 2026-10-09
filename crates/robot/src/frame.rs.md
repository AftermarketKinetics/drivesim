# frame.rs

## Function

Converts the STEP's axes to the robot frame (x forward, y left, z up),
because generic STEPs disagree on which way is up.

## Interface

- `axis("+x" | "-y" | "z" …) -> Option<[f64; 3]>`.
- `step_to_robot(up, forward) -> Option<M3>`: rows are the robot's x, y,
  z in STEP axes; `None` unless the two are perpendicular.

## Implementation

y is `up × forward`, so the frame is right-handed whatever the inputs.

## Assertions

- Tested: identity for `+z`/`+x`; a Y-up export turned upright; parallel
  or unparseable axes rejected.
