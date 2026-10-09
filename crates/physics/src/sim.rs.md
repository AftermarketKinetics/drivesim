# sim.rs

## Function

The fixed-step loop (docs/drivesim.md §6):
- physics at `PHYSICS_HZ`;
- control at `CONTROL_HZ`;
- motor torques applied every physics step.

## Interface

- `PHYSICS_HZ` 4000, `CONTROL_HZ` 1000, `STEPS_PER_TICK` 4, `GRAVITY`.
- `Sim { world, robot, model, target, duties }`:
  - `Sim::new(model, mesh)`: the arena, the box at (1, 0) and the robot
    at (−1, 0) facing +x;
  - `empty_world(gravity_z)` and `assemble(...)`, for tests that build
    their own scene;
  - `control_tick(frame)`, `physics_step()`;
  - `rotor_speed(rotor)`, `chassis()`.

## Implementation

- **Why 4 kHz:** a 10,000 RPM weapon on a rapier revolute joint throws
  the robot at a 1 ms step, whatever the substep count, and is stable
  from 2 kHz up. 4 kHz turns it 15° per step (lesson
  `rapier-fast-weapon-needs-small-dt.md`). Two solver iterations.
- **Motors use torques, not rapier joint motors.** The DC model's torque
  for the rotor's present relative speed is applied to the rotor about
  its world axis, with the opposite torque on the chassis, so the
  reaction torque (a robot twisting as its weapon spins up) is real. In
  the timing spike, rapier velocity motors on the wheels never moved the
  robot.
- **Duties update once per control tick** and are held across its four
  physics steps.
- **Momentum's firmware control (`kind = "momentum-core"`) isn't built
  yet;** such a robot sits idle.

## Assertions

- `PHYSICS_HZ` is a multiple of `CONTROL_HZ`.
- Tested (tests/drive.rs):
  - full speed within 5% of no-load;
  - turning the right way;
  - disarmed robots stay still;
  - the weapon reaches no-load speed and the robot stays upright;
  - driving pushes the box.
