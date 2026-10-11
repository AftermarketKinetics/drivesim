# camera.rs

## Function

The driver's-eye view: fixed outside the cage at standing eye height, as
a driver stands at an event. It is the only view in v1 (user,
2026-10-09).

## Interface

- `EYE_HEIGHT_M` = 1.7, `BEHIND_WALL_M` = 1.8, `FOV_DEG` = 58.
- `spawn_camera(commands, root)`: a `FOV_DEG` perspective camera behind the −x
  wall, looking at the cage's centre, z up.

## Implementation

- **Bloom** (`Bloom::NATURAL`, which brings an HDR target; user,
  2026-10-11: "make the impact marks glow more"). Anything emissive
  past 1 halos: the impact marks, and the arena's lamps too.

- Placed in rapier coordinates under `root`, like everything else, so
  "up" is `Vec3::Z`.
- The robot starts at (−1, 0) facing +x (physics `sim.rs`), which is
  facing away from the driver. Stick forward then drives away from the
  camera, as from the driver's box.

- **The whole floor is in view.** First the view widened from 50° to 58°
  (user, 2026-10-10). Then the camera moved back from 1.0 m to 1.8 m
  behind the wall (user, same day): at 1.0 m the bottom edge of the view
  met the floor 0.24 m inside the near wall, so a robot driven up to it
  disappeared. At 1.8 m that edge meets the floor 1.44 m in front of the
  camera, outside the cage.

## Assertions

- The camera never moves; there is no chase or orbit view in v1.
- The bottom of the view meets the floor outside the near wall: the
  whole cage floor is visible.
