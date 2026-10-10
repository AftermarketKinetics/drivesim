# sim.rs

## Function

The fixed-step loop (docs/drivesim.md §6):
- physics at `PHYSICS_HZ`;
- control at `CONTROL_HZ`;
- motor torques applied every physics step.

## Interface

- `PHYSICS_HZ` 4000, `CONTROL_HZ` 1000, `STEPS_PER_TICK` 4, `GRAVITY`.
- `Sim { world, robot, model, opponent, duties }`:
  - `Sim::new(model, mesh)`: the arena, the robot at (−1, 0) facing +x,
    and an opponent copy of the same model at (1, 0) facing it;
  - `empty_world(gravity_z)` and `assemble(world, robot, model,
    opponent)`, for tests that build their own scene (with `None`, no
    opponent);
  - `control_tick(frame)`, `physics_step()`;
  - `rotor_speed(rotor)`, `chassis()`, `opponent_chassis()`.
- `Opponent { robot }`, and `OPPONENT_DUTIES`: the opponent is armed
  with its sticks centred, so its wheels brake and its weapon is off.

## Implementation

- **Why 4 kHz:** a 10,000 RPM weapon on a rapier revolute joint throws
  the robot at a 1 ms step, whatever the substep count, and is stable
  from 2 kHz up. 4 kHz turns it 15° per step (lesson
  `rapier-fast-weapon-needs-small-dt.md`).
- **Contact as exact as rapier offers** (user, 2026-10-10, for about
  1.7× the physics cost):
  - Coulomb friction per contact point, not rapier's default single cone
    plus a "twist" constraint per patch;
  - no contact recycling, whose reused points go stale on a spinning
    wheel;
  - stiff floor contact springs, 200/400 Hz and critically damped,
    instead of the default 30/60 Hz with ζ = 10, which is tuned for
    piles;
  - 4 solver iterations, with joints warm-started.

  Measured against the defaults, grip itself didn't change: no creep
  under a 15 N side push, corners letting go at ~0.6–0.9 g, braking at
  0.67 g. What did improve is a full-stick spin in place: it wanders
  0.08 m instead of 0.34 m.
- **Motors use torques, not rapier joint motors.** The DC model's torque
  for the rotor's present relative speed is applied to the rotor about
  its world axis, with the opposite torque on the chassis, so the
  reaction torque (a robot twisting as its weapon spins up) is real. In
  the timing spike, rapier velocity motors on the wheels never moved the
  robot.
- **Each robot has its own `Drivetrain`** (`drivetrain.rs`), which
  applies its motor and gear-coupling torques before each world step and
  records its accelerations after. The opponent's runs with
  `OPPONENT_DUTIES`.
- **The opponent copy** replaced the target box (user, 2026-10-10). It
  shares the driven robot's model, and so its mass, gearing and
  colliders. It holds its ground on braked wheels, like an idle robot,
  rather than rolling away freely. It roughly doubles the physics cost:
  native 2.8× real time alone, 1.9× with it.
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
  - driving shoves the opponent;
  - full turn reaches 2 v / track; wheels roll without ringing; slamming
    the throttle does not flip it.
