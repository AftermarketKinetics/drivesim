# sim.rs

## Function

The fixed-step loop (docs/drivesim.md §6):
- physics at `PHYSICS_HZ`, or another of `PHYSICS_RATES_HZ` set at run
  time;
- control at `CONTROL_HZ`;
- motor torques applied every physics step.

## Interface

- `PHYSICS_HZ` 2000 (the rate a `Sim` starts at), `PHYSICS_RATES_HZ`
  [2000, 3000, 4000], `CONTROL_HZ` 1000, `GRAVITY`.
- `OpponentKind { Copy (default), Box, Ai, AiVsAi }`: what the robot
  faces. `AiVsAi` is `Ai` with an AI driving the robot as well.
- `Rival<'a> { model, mesh }`: another robot to face in place of the
  copy.
- `Sim { world, robot, model, opponent, target, duties, time_s }`:
  - `Sim::new(model, mesh, kind)`: the arena, the robot at (−1, 0)
    facing +x, and at (1, 0) either an opponent copy of the same model
    facing it (`opponent`, with an `AiDriver` for `Ai`) or the box from
    `target.rs`, as heavy as the robot (`target`). Exactly one of the
    two is set;
  - `Sim::versus(model, mesh, kind, rival)`: `new`, with `rival` (if
    `Some`) spawned at (1, 0) in place of the copy for `Copy` and `Ai`;
    `Box` ignores it. `new` is `versus(.., None)`;
  - `empty_world(gravity_z)` and `assemble(world, robot, model,
    opponent: Option<Opponent>)`, for tests that build their own scene
    (with `None`, no opponent);
  - `control_tick(frame)`, `physics_step()`. A tick runs
    `physics_hz() / CONTROL_HZ` steps;
  - `physics_hz()` and `set_physics_hz(hz)`: the step rate, read from the
    world's `dt`. `set_physics_hz` takes `hz` to the nearest of
    `PHYSICS_RATES_HZ` and applies from the next tick (user, 2026-10-11:
    the sim page's 2–4 kHz slider);
  - `time_s`: simulated seconds since the round started;
  - `floor`: the arena floor collider, set by `new`/`versus`. While it
    is set, every physics step runs `impact_loss.rs`. `None` (a scene
    from `assemble`, or a test setting it) turns the loss off;
  - `countdown_left()`: seconds until the AI starts, `None` without an
    AI or once it has;
  - `impacts` (at most `MAX_IMPACTS` 256, oldest dropped) and
    `take_impacts()`: the large impacts since the last take
    (`impacts.rs`), for the page's glowing marks;
  - `pilot`: `Some(AiDriver)` in AI vs AI. Each control tick it drives
    the robot in place of the radio: `OPPONENT_DUTIES` (braked) until
    the countdown ends, then it chases the opponent's chassis, weapon at
    full;
  - `bout_left()`: seconds left in the AI bout (`BOUT_S` through the
    countdown, then down to 0 from FIGHT), `None` without an AI. At 0
    every control tick gives both robots no pulses, so they coast down
    and stay disarmed until the round is rebuilt (user, 2026-10-11: "run
    the vs ai in 3 minute bouts", ending with TIME and a rematch);
  - `rotor_speed(rotor)`, `chassis()`, `opponent_chassis()` (the
    copy's chassis or the box's body).
- `Opponent` and `OPPONENT_DUTIES` live in `opponent.rs`.

## Implementation

- **Why 2 kHz:** a 10,000 RPM weapon on a rapier revolute joint throws
  the robot at a 1 ms step, whatever the substep count, and is stable
  from 2 kHz up, 30° per step (lesson
  `rapier-fast-weapon-needs-small-dt.md`). It ran at 4 kHz (15° per
  step) until the user chose 2 kHz for speed (2026-10-10): Obliterator 6
  v72 against its AI copy went from 1.31× to 2.4× real time natively
  (0.7× in the browser before). Every drive and weapon test still
  passes; the chaotic single-hit test's margin shrank (see
  tests/drive.rs.md). **2 kHz isn't enough for every robot:** a 0.96 kg
  beetle's 45 mm bar at 10.6k rpm blew up in its hits at 2 and 3 kHz,
  and held at 4 (2026-10-11). The user chose a 2–4 kHz slider, defaulting
  to 2, over raising the rate for everyone. **Weapon CCD stays on:** dropping it too reached
  3.6×, but the spinning fixture then hopped on open floor and hits
  stopped throwing the opponent.
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
- **Contact prediction margin of 5 mm**, not rapier's default 20 mm
  (`normalized_prediction_distance` 0.005; 2026-10-10). Every hull within
  the margin of another surface gets a speculative manifold in the
  solver. Obliterator 6 v72 (a low robot) had 19 hulls within 20 mm of
  the floor: 83 solver contacts at rest instead of about 17. At 2 kHz a
  body at 10 m/s moves 5 mm per step, and the weapon has real CCD. 2 mm
  saved little more, but it flipped the outcome of the chaotic
  `a_hit_throws_the_opponent_up_not_the_robot` test. Lesson
  `rapier-default-prediction-distance-is-20mm.md`.

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
  records its accelerations after. The opponent's runs with its own
  `duties`, which `Opponent::control` sets once per control tick:
  `OPPONENT_DUTIES`, or the AI's after the countdown. The opponent's
  drivetrain runs on the opponent's own `model`, so a rival brings its
  own motors, gearing and battery.
- **The opponent copy** replaced the target box (user, 2026-10-10). It
  is a clone of the driven robot's model, and so has its mass, gearing
  and colliders. It holds its ground on braked wheels, like an idle robot,
  rather than rolling away freely. It roughly doubles the physics cost:
  native 2.8× real time alone, 1.9× with it.
- **The box is the other choice** (user, 2026-10-10: a toggle on the sim
  page). It is the box the copy replaced, restored as `target.rs`; it has
  no drivetrain, so `physics_step` only touches it through the world.
  `assemble` never sets it: `new` adds it after, so tests that build
  their own scene are unchanged.
- **Impacts are lossy** (user, 2026-10-11): `physics_step` snapshots
  each robot's (and the box's) body velocities before `world.step()`,
  then `impacts::find_impacts` records the large ones and
  `impact_loss::absorb` scales back any group's impact. The groups
  are rebuilt each step from the handles, which is cheap next to the
  step itself.
- **Duties update once per control tick** and are held across its four
  physics steps, the driven robot's and the opponent's alike.
- **AI vs AI** (user, 2026-10-11: "takes over control of the bot from
  you, allowing you to test the bot without an elrs set up") reuses the
  opponent's `AiDriver` and `pose_of` for the robot. Its frame is
  ignored, and so is the mixer's arm switch. The bout clock and the
  countdown key off the opponent's AI, so they behave as in AI.
- **The AI opponent** (user, 2026-10-10) is the copy with an
  `AiDriver`; it waits `COUNTDOWN_S` of sim time, then chases with its
  weapon at full. `time_s` counts from `new`/`assemble`, so a rebuilt
  round restarts the countdown.
- **A rival** (user, 2026-10-10) is a public robot picked for the AI on
  the sim page. It goes through the same `spawn_robot` at the same spot
  as the copy, with its own mesh closure, and becomes an `Opponent` with
  its own model. `versus` doesn't check the weight class: picking a
  mismatched opponent is allowed.
- **Momentum's firmware control (`kind = "momentum-core"`) isn't built
  yet;** such a robot sits idle.

## Assertions

- `PHYSICS_HZ` and every rate in `PHYSICS_RATES_HZ` are multiples of
  `CONTROL_HZ`, so a tick is always 1 ms of simulated time.
- Tested (tests/drive.rs):
  - full speed within 5% of no-load;
  - turning the right way;
  - disarmed robots stay still;
  - the weapon reaches no-load speed and the robot stays upright;
  - (tests/impact.rs) impacts lose energy: hits and wall recoils are
    slower than without the loss;
  - driving shoves the opponent, copy or box; the box spawns at (1, 0)
    with the robot's mass;
  - (tests/ai.rs) a rival keeps its own model and mass and still
    reaches the robot;
  - full turn reaches 2 v / track; wheels roll without ringing; slamming
    the throttle does not flip it.
