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
