# index.html

## Function

The page shell for the browser app. It holds the canvas, starts the wasm,
and says plainly when the browser can't run it (docs/drivesim.md §8).

## Interface

- It is opened with `?robot=<id>` (drivesim-server) or
  `?bundle=<url>&motors=<url>` (a static bundle); `bundle_load.rs` reads
  these.
- `<canvas id="drivesim">`, the canvas `main.rs` binds its window to.
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
- The loading message goes once the canvas has been sized, which is
  Bevy's first frame.

## Assertions

- The canvas id matches `main.rs`, and the module name matches the
  Makefile's `--out-name`.
