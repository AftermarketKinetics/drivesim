# lib.rs (drivesim-robot)

## Function

Turns a robot bundle (the importer's `parts.json`, the role
conversation's `robot.toml`, the motor library) into a `RobotModel` the
physics builds bodies from (docs/drivesim.md §5).

## Interface

Modules `frame`, `glob`, `load`, `mass`, `materials`, `math`, `model`,
`motors`, `parts`, `robot_file`, `spin`, `stl`; re-exports `load`, `LoadError`,
`RobotModel`, `Body`, `Rotor`, `Side`.

## Implementation

`load` is the one entry point; the rest are its pieces, each one job.
Dependencies are serde, serde_json and toml only: all build for wasm32.

## Assertions

- Text and bytes in, never file paths.
- Everything past `load` is in the robot frame (x forward, y left, z up)
  and SI units.
