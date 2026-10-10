# drivesim bootstrap (session started in ~/code/momentum)

Earlier turns of this session (Momentum's 2D sim, the Pocket input, the
decision to make the sim a general tool in its own repo) are logged in
`~/code/momentum/.claude/chat/2026-10-08-driving-sim-start.md`.

## 05:45 Repo setup and design revision
**User:** go (then: browser on accelerate v1, server-side STEP import, own gVisor broker, Chrome/Edge, localhost dev)
**Did:**
- shell.nix (momentum's pin, pythonocc 7.9, wasm-bindgen 0.2.127, binaryen), CLAUDE.md, sidecar.json, Makefile; importer + fixture + 7 tests, sidecars for every file.
- docs/drivesim.md rewritten for the browser target (server, broker, role conversation via API, bytes-not-paths crates, build order §11).

## 06:15 Browser spike
**Did:**
- tools/padtest/index.html (raw Gamepad API page) + tools/dev_server.py (localhost:8417, COOP/COEP) + `make web-dev`; added lld to shell.nix for wasm32 linking.
- rapier3d timing spike (scratchpad/rbench): 10k RPM weapon joint unstable at 1 ms whatever the substeps, stable from 2 kHz; WASM (V8) 6.6x realtime at 4 kHz. Lesson rapier-fast-weapon-needs-small-dt.md; doc §6/§10 updated (proposal: 4 kHz physics). Wheel motors didn't move the robot: open.

## 06:40 Pad test and timing in Chrome
**User:** connected: Radiomaster Pocket Joystick (Vendor: 1209 Product: 4f54) / pad 0: id="Radiomaster Pocket Joystick (Vendor: 1209 Product: 4f54)" mapping="" axes=8 buttons=24 all the mappings look goiod  (+ pasted timing page: 4 kHz 5.4–6.1x realtime, stable; 1 kHz x4 unstable)
**Did:**
- Recorded both in docs/drivesim.md §6/§7. New crates/input (frame_from_axes, EdgeTX USB-ID match), 9 tests; `make test` now runs cargo test/clippy/fmt.

## 07:10 robot crate
**Did:**
- crates/robot: parts.json/robot.toml/motors.toml parsing, frame change, densities, mass rollup, loader with named errors; data/motors.toml (2 motors, sourced, placeholders flagged); fixtures/simple_bot bundle + `make fixtures`. 21 tests, 24/24 sidecars paired.

## 08:00 physics crate
**Did:**
- crates/physics (rapier3d 0.36): motor model, mixer, hull-or-VHACD shapes, arena + box, spawn (shared robot frame, model mass props, cylinder wheels, revolute joints), 4 kHz/1 kHz sim with motors as equal-and-opposite torques.
- 17 tests pass: precession = tau/L within 10%, angular momentum within 1%, 97.6% of no-load speed, turning, disarm, weapon spin-up, box push. First failures were test bugs (box in the path; NaN mass props before first step).
- Doc §6/§9/§10 updated; 34/34 sidecars; make test green (clippy/fmt clean).

## 09:30 Commit, next step, server questions, server built
**Asked:** Next? Web app first (Recommended) / Server first (as planned) / Momentum-core hook first → "Server first (as planned)". Commit? → "Commit both". Reuse? Copy and adapt (Recommended) / Shared library / Write fresh → "Copy and adapt (Recommended)". Model? claude-sonnet-5-5 (Recommended) / claude-opus-5-5 / claude-haiku-4-5 → "claude-sonnet-5-5 (Recommended)". API key? Its own key / Share the analyzer's key → "Share the analyzer's key". Sharing? Owner only, for v1 / Everyone signed in / Owner + explicit shares → "Owner only, for v1".
**Did:**
- Committed drivesim ad9d5ec and momentum eee7123 (momentum's 2D sim removed). Added physics `rest` + crates/cli drivesim-check.
- server/ (FastAPI, uv: anthropic 1.13) copied/adapted from the analyzer: store, auth (+dev user), events, llm, loop, 4 tools, prompt, importer (broker|local), pages; broker/ (one fixed gVisor docker run). 38 server + 5 broker tests pass; 63/63 sidecars.

## 10:00 Live role conversation
**Asked:** Live test? Yes, run it / You run it / Later → "Yes, run it". Commit? Commit now / After the live test / Not yet → "Commit now". Next? Yes, web app next / Deploy images first / Stop here for now → "Yes, web app next".
**Did:**
- Committed d14fef7. Ran one real conversation (sonnet-5-5 via the ant profile) on the fixture through server-dev: correct roles from geometry, one batched question set, checked robot.toml in two turns.
- Tuned from it: notice carries the upload name, tilt on an edge is normal, arm = 5 by default. 38 tests pass.

## 10:20 Web app plan questions
**Asked:** Arena? Keep the placeholder / I'll give the size / Look it up → "Look it up". Camera? Driver's eye only / Driver's eye + chase toggle / Driver's eye + overhead → "Driver's eye only".
**Did:**
- NHRL wiki (Facilities, Cage Design; edited 2025-08): 12/30 lb cage 16'x16', painted plywood floor (not steel), double-walled polycarbonate on a timber skirt, roofed. arena.rs/doc/prompt updated; wall height 2.4 m provisional.
- Started web/ (Bevy 0.20, own workspace, minimal features: no gilrs/multithread/audio, webgpu); cold wasm build of a cube spike running.

## 11:00 Web app first build
**Did:**
- web/ app: radio (Gamepad API by EdgeTX ID), bundle_load (fetch + same loader), sim_step (real time, 50 ms catch-up cap), robot_render (per-part meshes, z-up root), arena_render (16 ft cage, box), camera (driver's eye), hud; page shell; `make web-build` -> server/web/sim (48 MB wasm after wasm-opt).
- Bevy 0.20 builds for wasm32 with the nix toolchain (deps 6.5 min cold); two API renames fixed. Not yet run in a browser: asked the user to open the fixture in Chrome.

## 2026-10-10 02:10 Web app renders; sidecars, wasm size
**User:** it renders fine, continue building
**Did:**
- Sidecars for every `web/` file; `web` added to `.claude/sidecar.json` roots (build output `server/web/sim/` ignored).
- HUD shows physics headroom (× real time, `performance.now()` around stepping) so build changes can be judged in the browser.
- Wasm 48.5 MB (43.5 MB code, 11.6 MB gzipped): trying opt-level "s" + fat LTO with the physics stack at opt 3; the opt-3 build kept at `out/sim-o3/` to compare.

## 2026-10-10 02:35 Size build done
**Did:**
- opt "s" + fat LTO (physics stack at opt 3) + `wasm-opt -Os`: 48.5 → 30.0 MB (11.6 → 8.6 MB gzipped); release build now ~13 min.
- `server/web/robot.html` links checked robots to `sim/?robot=<id>`.
- Asked the user to compare physics headroom between `out/sim-o3/` and `server/web/sim/`.

## 2026-10-10 02:50 First drive: turning too fast
**User:** can push the box, controls work, SA arms, weapon spins up. main issue is that the friction/turning is way undetuned, you turn way too fast. forward acceleration is good
**Did:**
- Probe (temporary test, deleted): full-stick spin reaches 160 rad/s, 95–97% of the ideal 2v/track at every stick level, so friction is not losing anything; the fixture's invented gearing gives 16.9 m/s top speed (Momentum's target is 4.47 m/s on a 240 mm track, so ~37 rad/s).
- Asking the user how to fix it (fixture gearing, turn shaping, scrub model) before changing anything.
