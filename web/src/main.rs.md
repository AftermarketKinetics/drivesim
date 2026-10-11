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
  (the `SimRoot` entity, read by `opponent_toggle`). Startup also inserts
  `rival_load::Rivals` with the page's motor library URL, when the URL
  locates a bundle.
- Update systems run chained, in this order: `poll_radio`, `take_bundle`,
  `apply_opponent_toggle`, `step_sim`, `sync_bodies`, `update_marks`, `update_hud`,
  `update_countdown`, `update_bout_clock`.
- Startup runs `impact_marks::setup_marks` beside `setup`, which
  spawns the HUD, the countdown banner (`countdown.rs`) and
  the bout clock (`bout.rs`).
- The wasm module exports `set_opponent` (`opponent_toggle.rs`) for
  the page's opponent buttons and `set_rival` (`rival_load.rs`) for its
  AI robot dropdown, plus `bout_over` and `rematch` (`bout.rs`) for its
  Rematch button and `set_marks` (`impact_marks.rs`) for its Marks
  checkbox.

## Implementation

- **One rotation turns rapier into Bevy.** Rapier's world is z up and
  Bevy's is y up; `SimRoot` is rotated −90° about x and every simulated or
  arena entity is its child. Poses therefore copy across unchanged.
- **Loading is async but the App is not.** `setup` starts `fetch_bundle`
  with `spawn_local`, which leaves its result in `LoadSlot`. `take_bundle`
  polls the slot each frame; on success it builds the first round
  (`round.rs`) with the opponent the page asks for (the AI on its own
  copy, if a rival is wanted: `apply_opponent_toggle` swaps the rival in
  once its bundle loads), and keeps the bundle
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
