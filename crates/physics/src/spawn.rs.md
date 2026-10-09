# spawn.rs

## Function

Turns a `RobotModel` into rapier bodies, colliders and revolute joints
(docs/drivesim.md §6).

## Interface

- `spawn_robot(world, model, mesh, x, y, yaw) -> Result<RobotHandles,
  SpawnError>`: `mesh(path)` fetches a bundle file. The robot's lowest
  point is placed on the floor.
- `RobotHandles { bodies, wheel_joints, weapon_joints }`: `bodies` runs
  parallel to `model.bodies`, with `[0]` the chassis.
- `SpawnError { MissingMesh, BadMesh }`.
- Helpers: `v` (f64 array → `Vector`), `forward_sign(wheel)`,
  `side_of(wheel)`, `BODY_MU` (0.3).

## Implementation

- **Every body is created at the same pose,** with its local frame equal
  to the robot frame. Model coordinates (CoMs, axis points) are then
  local coordinates as they stand, and a joint's anchor is the same
  point on both bodies.
- Mass properties are set from the model with `with_inertia_matrix`;
  colliders have zero density, so they add no mass.
- **Wheel colliders:** a cylinder at the measured radius and width,
  rotated from rapier's y axis onto the wheel's axis. A tessellated rim
  would bump.
- **Other bodies:** one `part_shape` per part. The weapon gets CCD.
- **Joints:** revolute, with contacts off between the jointed pair.
- **Placement:** rests the lowest mesh vertex or wheel rim on the floor,
  with 0.5 mm clearance. The rim calculation assumes a horizontal axle.

## Assertions

- `forward_sign`: positive spin about an axle with positive robot-y
  rolls the robot forward (the hub moves along `axis × z`).
