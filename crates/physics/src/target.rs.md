# target.rs

## Function

The box opponent (docs/drivesim.md §6): a free box the driver can pick
instead of the robot's own copy, with the sim page's opponent button.

## Interface

- `BOX_HALF_M` [0.15, 0.15, 0.075] (300 × 300 × 150 mm), `BOX_MU` 0.4.
- `add_target(world, x, y, mass_kg) -> RigidBodyHandle`: a dynamic box
  resting on the floor at (x, y).

## Implementation

- It is the box commit 495eb8a removed when the opponent copy took over,
  restored unchanged (user, 2026-10-10: "change the opponent from an
  orange version of your bot to an orange box"). `Sim::new` gives it the
  robot's total mass, so pushing it feels like pushing an opponent of
  the same weight class.
- Friction combines by multiplying, like every robot collider: rapier
  uses the stronger of two colliders' combine rules, and the floor's
  1.0 x 0.4 makes it slide on 0.4.
- It has no drivetrain and no weapon; it is drawn orange by the web app.

## Assertions

- The box's bottom face rests at z = 0 at spawn.
- Tested (tests/drive.rs): it spawns at (1, 0) with the robot's mass,
  and full throttle shoves it.
