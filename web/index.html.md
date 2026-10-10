# index.html

## Function

The page shell for the browser app. It holds the canvas, starts the wasm,
and says plainly when the browser can't run it (docs/drivesim.md §8).

## Interface

- It is opened with `?robot=<id>` (drivesim-server) or
  `?bundle=<url>&motors=<url>` (a static bundle); `bundle_load.rs` reads
  these.
- `<canvas id="drivesim">`, the canvas `main.rs` binds its window to.
- `<button id="opponent">`, top right: "Opponent: copy" / "Opponent:
  box". Each click flips the choice and calls the wasm export
  `set_opponent_box` (`opponent_toggle.rs`), which restarts the round.
- It imports `./drivesim_web.js`, the name `make web-build` gives
  wasm-bindgen's output. `make web-build` copies this file next to it in
  `server/web/sim/`.

## Implementation

- **WebGPU only:** without `navigator.gpu` it shows a message instead of
  loading ~30 MB of wasm that cannot run (Chrome/Edge only, user,
  2026-10-09).
- winit hands control to the browser's event loop by throwing a "Using
  exceptions for control flow" error out of `init()`. That one is
  ignored; any other error is shown.
- The loading message goes, and the opponent button appears, once the
  canvas has been sized, which is Bevy's first frame. Before that the
  wasm export doesn't exist.
- **The button hands focus back to the canvas** after each click, so the
  keyboard never stays on it (user, 2026-10-10: an HTML button rather
  than a key or URL parameter).
- The page holds the button's state itself; the wasm only hears the
  latest choice, and starts on the copy, matching the button.

## Assertions

- The canvas id matches `main.rs`, and the module name matches the
  Makefile's `--out-name`.
- The button's label starts at "copy", matching `opponent_toggle`'s
  default.
