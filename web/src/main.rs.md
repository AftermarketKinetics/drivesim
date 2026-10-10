# main.rs

## Function

The browser app's entry point (docs/drivesim.md §8): it fetches a robot
bundle, simulates it with drivesim-physics, reads the radio through the
Gamepad API, and draws everything from the driver's box.

## Interface

- `main()`, called by wasm-bindgen's `init()` from `index.html`.
- The window is bound to the page's `<canvas id="drivesim">` and fits its
  parent.
- Resources it owns: `LoadSlot` (the async fetch's result) and `Root`
  (the `SimRoot` entity, read by `opponent_toggle`).
- Update systems run chained, in this order: `poll_radio`, `take_bundle`,
  `apply_opponent_toggle`, `step_sim`, `sync_bodies`, `update_hud`.
- The wasm module exports `set_opponent_box` (`opponent_toggle.rs`) for
  the page's button.

## Implementation

- **One rotation turns rapier into Bevy.** Rapier's world is z up and
  Bevy's is y up; `SimRoot` is rotated −90° about x and every simulated or
  arena entity is its child. Poses therefore copy across unchanged.
- **Loading is async but the App is not.** `setup` starts `fetch_bundle`
  with `spawn_local`, which leaves its result in `LoadSlot`. `take_bundle`
  polls the slot each frame; on success it builds the first round
  (`round.rs`) with the opponent the page asks for, and keeps the bundle
  as `Loaded` so `apply_opponent_toggle` can rebuild without refetching.
  On failure the HUD shows the reason. Systems that need the sim take `Option<Res<SimRes>>` and do
  nothing until it exists.
- **The order matters:** a switched round is built before it is stepped;
  the radio is read before the step that uses it,
  and bodies are synced after the step that moved them.

## Assertions

- `poll_radio` runs before `step_sim`, and `step_sim` before `sync_bodies`.
- Every entity drawn in rapier coordinates is a child of the `SimRoot`
  entity.
- A failed load never panics; it shows on the HUD.
