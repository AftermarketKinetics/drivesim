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
  `side_of(wheel)`, `reflected_inertia(rotor)`, `BODY_MU` (0.3).

## Implementation

- **Every body is created at the same pose,** with its local frame equal
  to the robot frame. Model coordinates (CoMs, axis points) are then
  local coordinates as they stand, and a joint's anchor is the same
  point on both bodies.
- Mass properties are set from the model with `with_inertia_matrix`;
  colliders have zero density, so they add no mass.
- **Each rotor body also gets its motor's reflected inertia,**
  `rotor_inertia_kg_m2 · ratio²`, about its axis (added as `J a aᵀ`). It
  is real (the gearbox spins the motor's rotor ratio times faster), and
  it is what keeps the explicit motor torque stable at the physics step
  (0.5 ms since 2 kHz, 2026-10-10; it was 0.25 ms). Without it,
  a 68 g wheel at 22.5:1 has a motor time constant of ~0.1 ms, under
  the step, and its speed rings between the current limits
  (found 2026-10-10 when the fixture was regeared). The lump overstates
  the rotor's angular momentum about `ratio` times, so wheel bodies have
  rapier's gyroscopic term off. `gear_coupling.rs` corrects its reaction
  on the chassis.
- **Wheel colliders:** a cylinder at the measured radius and width,
  rotated from rapier's y axis onto the wheel's axis. A tessellated rim
  would bump.
- **Other bodies:** `body_shapes`, the convex hulls of the body's 12
  biggest parts (`shapes.rs`). The weapon gets CCD, and collides only
  through the parts that reach near its outer radius
  (`weapon_reach::reaching_parts`, user 2026-10-10); its internals stay
  in its mass.
- **Joints:** revolute, with contacts off between the jointed pair.
- **Placement:** rests the lowest mesh vertex or wheel rim on the floor,
  with 0.5 mm clearance. The rim calculation assumes a horizontal axle.

## Assertions

- `forward_sign`: positive spin about an axle with positive robot-y
  rolls the robot forward (the hub moves along `axis × z`).
