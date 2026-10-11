# drivetrain.rs

## Function

One robot's motors in the world (docs/drivesim.md §6). Each physics
step, every rotor gets its motor's torque for its present speed, equal
and opposite on the chassis, plus the gearbox coupling torques
(`gear_coupling.rs`). `Sim` keeps one `Drivetrain` per robot: the driven
robot's and the opponent copy's.

## Interface

- `Drivetrain` (`Default`): the motor state carried between steps.
  - `apply(world, model, robot, duties)`: before a world step, resets the
    robot's torques and adds this step's motor and coupling torques. The
    coupling torques are scaled by `gear_coupling::coupling_scale`,
    worked out from the model on the first call (2026-10-11).
  - `observe(world, model, robot)`: after a world step, records the
    accelerations over it for the next step's coupling.
- `rotor_speed(world, robot, rotor) -> f32`: rotor speed relative to the
  chassis about its axis, rad/s.

## Implementation

- **Split out of `sim.rs`** on 2026-10-10, when the box became an
  opponent copy and the sim needed the same motor stepping for two
  robots.
- Rotors are walked wheels first, then weapons. The order indexes the
  carried state.
- **Wheels:** a wheel's duty comes from its side, and its speed and
  torque are signed by `forward_sign`, so a positive duty rolls the robot
  forward. A weapon's duty is the weapon duty, unsigned.
- **The coupling uses last step's accelerations,** from the speeds about
  each rotor's axis recorded by `observe`. On the very first step there
  are none yet, and the coupling is zero.

## Assertions

- `apply` and `observe` bracket every world step, for every robot.
- Torques act on the rotor and the chassis about the same world axis, in
  opposite directions; the motor never puts net torque into the robot.
