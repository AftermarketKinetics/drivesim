# camera.rs

## Function

The driver's-eye view: fixed outside the cage at standing eye height, as
a driver stands at an event. It is the only view in v1 (user,
2026-10-09).

## Interface

- `EYE_HEIGHT_M` = 1.7, `BEHIND_WALL_M` = 1.0, `FOV_DEG` = 58.
- `spawn_camera(commands, root)`: a `FOV_DEG` perspective camera behind the −x
  wall, looking at the cage's centre, z up.

## Implementation

- Placed in rapier coordinates under `root`, like everything else, so
  "up" is `Vec3::Z`.
- The robot starts at (−1, 0) facing +x (physics `sim.rs`), which is
  facing away from the driver. Stick forward then drives away from the
  camera, as from the driver's box.

- **Zoomed out by widening the view, not by moving the camera.** The
  driver stays where a driver stands. The field of view went from 50° to
  58° (user, 2026-10-10).

## Assertions

- The camera never moves; there is no chase or orbit view in v1.
