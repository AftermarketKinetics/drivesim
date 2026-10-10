# sim_step.rs

## Function

Runs the simulation in real time: control ticks at `CONTROL_HZ`, each
running `STEPS_PER_TICK` physics steps (docs/drivesim.md §6). It also
measures how much faster than real time the physics could run.

## Interface

- `SimRes { sim, headroom }`: a resource, inserted once the robot loads.
  - `SimRes::new(sim)`.
  - `headroom` is simulated seconds per wall-clock second spent stepping,
    smoothed; the HUD shows it as "physics N× real time".
- `MAX_CATCH_UP_S` = 0.05.
- `step_sim`: a system that feeds the current `Radio` frame to every
  control tick it runs.

## Implementation

- **An accumulator, not a fixed tick count per frame.** Frame time builds
  up in `behind_s`, and whole control ticks are run until it is spent.
  Frame-rate changes therefore don't change sim speed.
- **Catch-up is capped at 50 ms per frame.** A slower frame (a background
  tab, a GC pause) drops the rest. Otherwise one stall would run a burst
  of physics long enough to stall the next frame too.
- **Headroom** is timed with `performance.now()` around the stepping loop
  and smoothed (`HEADROOM_EMA`, 0.02 per frame). Below 1, the sim can't
  keep up and runs slow. It is the in-browser check that a build or
  profile change kept 4 kHz affordable.

## Assertions

- Every control tick gets the same radio frame within one Bevy frame.
- At most `MAX_CATCH_UP_S` of simulated time is added per frame.
