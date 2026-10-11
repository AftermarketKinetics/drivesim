# index.html

## Function

The page shell for the browser app. It holds the canvas, starts the wasm,
and says plainly when the browser can't run it (docs/drivesim.md §8).

## Interface

- It is opened with `?robot=<id>` (drivesim-server) or
  `?bundle=<url>&motors=<url>` (a static bundle); `bundle_load.rs` reads
  these.
- `<canvas id="drivesim">`, the canvas `main.rs` binds its window to.
- `<div id="opponent">`, top right: "Opponent" and three buttons,
  Copy / Box / AI / AI vs AI, the chosen one highlighted (`aria-pressed`,
  filled orange). Each button's `data-kind` (0, 1, 2, 3) is what it passes to
  the wasm export `set_opponent` (`opponent_toggle.rs`); choosing
  another restarts the round.
- `<select id="rival">` beside them, shown only while AI or AI vs AI is
  chosen (user,
  2026-10-10): "Your robot (copy)" (value "") and then every public
  robot from `GET /api/opponents` (minus the robot being driven), in an
  `<optgroup>` per weight class. The driven robot's class (its
  `weight_lb` from `GET /api/robots/{id}`) comes first, then the rest
  ascending. Option values are bundle base URLs
  (`/api/robots/<id>/bundle/`), passed to the wasm export `set_rival`
  (`rival_load.rs`). The caller's own robots are suffixed "(yours)".
  With a static `?bundle=` the list is the house robots' fixtures,
  `/fixtures/simple_bot/` (12 lb) and its `_1lb`, `_3lb` and `_30lb`
  versions, for `make web-dev`.
- `<button id="rematch">`, centred, hidden until an AI bout runs out
  (user, 2026-10-11). Every 250 ms the page polls the wasm export
  `bout_over` (`bout.rs`) to show or hide it. A click calls `rematch`,
  which rebuilds the round as it is.
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
- The loading message goes, and the opponent buttons appear, once the
  canvas has been sized, which is Bevy's first frame. Before that the
  wasm export doesn't exist.
- **Buttons hand focus back to the canvas** after each click, so the
  keyboard never stays on one (user, 2026-10-10: HTML buttons rather
  than a key or URL parameter).
- **A dropdown for the AI's robot** (user, 2026-10-10, asked by name):
  unlike the three opponent kinds, the list of public robots grows
  without bound, so it can't all be shown as buttons.
- The list is fetched once at load; if it fails, the dropdown keeps only
  the copy and the reason goes to the console.
- **Three buttons, not one cycling button** (user, 2026-10-10: "make
  the toggle more obvious"): every choice is visible and one click away.
- The page holds the highlight itself; the wasm only hears the latest
  choice, and starts on the copy, matching the highlighted button.
  Clicking the chosen one again calls `set_opponent` with the same
  value, which rebuilds nothing.

## Assertions

- The canvas id matches `main.rs`, and the module name matches the
  Makefile's `--out-name`.
- Copy starts highlighted, matching `opponent_toggle`'s default.
- `data-kind` values match `opponent_toggle::wanted`: 0 copy, 1 box,
  2 AI, 3 AI vs AI.
- The dropdown is visible exactly when AI or AI vs AI is highlighted, and its first
  option, "" (the copy), matches `rival_load`'s default.
