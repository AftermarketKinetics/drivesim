# physics_rate.rs

## Function

Holds the running simulation at the physics rate the page's "Physics"
slider asks for (user, 2026-10-11: "allow the user to select a slider
from 2 to 4 kHz", 2 by default). A light robot's small, fast weapon needs
4 kHz to stay physical in hits (docs/drivesim.md §6, Timestep).

## Interface

- `set_physics_khz(khz)` (wasm export): the slider's value, 2–4.
- `apply_physics_rate`: an Update system, chained before
  `sim_step::step_sim`. It sets the `SimRes` sim's rate when it differs
  from the wanted one.

## Implementation

- **A static (`AtomicU32`), not a resource,** because the call comes in
  from JS outside Bevy's schedule, as with `impact_marks::set_marks`.
- **Applied every frame, not once:** `round.rs` builds a fresh `Sim` at
  `PHYSICS_HZ` on each opponent change and rematch, and the next frame
  puts it on the slider's rate. Comparing first keeps the call off the
  hot path.
- **The snapping is `Sim::set_physics_hz`'s:** it takes any value to the
  nearest of `PHYSICS_RATES_HZ`. This file passes the kHz through
  unchecked.
- It starts at `PHYSICS_HZ`, so before the page's first call, and in a
  page without the slider, the sim runs at the default.

## Assertions

- `apply_physics_rate` runs before `step_sim` in the same frame, so a
  slider move never splits a frame's ticks across two rates.
- Every `Sim` the page steps is at the wanted rate from its first
  stepped frame.
