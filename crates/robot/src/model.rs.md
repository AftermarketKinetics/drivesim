# model.rs

## Function

The loaded robot, in the robot frame and SI units: what the physics
crate builds rapier bodies and joints from.

## Interface

- `RobotModel { name, battery_v, step_to_robot, bodies, wheels, weapons,
  control }`; `total_mass()`, `combined()`.
- `Body { name, parts, meshes, mass }`: `bodies[0]` is the chassis.
- `Rotor { name, body, axis_point, axis_dir, radius_m, width_m, motor,
  ratio, efficiency, mu, side }`: `mu` and `side` are `Some` for wheels
  only.
- `Side { Left, Right }`.

## Implementation

Meshes are referenced by bundle-relative path, not loaded: the caller
fetches them (from disk in tests, over HTTP in the browser) and applies
`step_to_robot` to their vertices.

## Assertions

- `bodies[0]` is always the chassis; every `Rotor::body` indexes
  `bodies`.
