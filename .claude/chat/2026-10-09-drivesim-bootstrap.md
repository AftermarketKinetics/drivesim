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

## 2026-10-10 03:40 Regear, ringing fix
**Asked:** fixture gearing → "Momentum-like gearing (Recommended)"; turn shaping → "On the radio, as at an event (Recommended)"; tyre scrub → "Not now (Recommended)"; headroom/commit → "Same speed; commit".
**Did:**
- Committed the web app (0898b59). Fixture regeared to 22.5:1 (4.5 m/s, 44 rad/s full turn); this exposed wheel speed ringing at the current limits. Fixed by adding motor rotor inertia × ratio² to rotor bodies (`spawn.rs`, provisional 5e-6 kg m² in `data/motors.toml`) with gyroscopic forces off on wheels; lesson `explicit-motor-torque-needs-reflected-inertia.md`.
- New drive tests: full-turn yaw = 2v/track, wheels roll without ringing; straight test now at 40%. Fixture backflips at full throttle from rest (CoM over axle, 2 × 5.9 N m), reported to the user.

## 2026-10-10 04:20 Flips removed; turning still strong
**User:** turning is still way too strong, flicking forward and backwards causes frontflips and backflips, without weapon spinup, remove them
**Did:**
- Cause: the lumped reflected inertia (J·r² on the wheel) returned ~22× too much spin-up reaction to the chassis. New `crates/physics/src/gear_coupling.rs` adds the real gearbox coupling torques each step (from last step's accelerations); full/reverse/full slams now keep up.z > 0.99. Test `slamming_throttle_does_not_flip_it`; sidecars, doc §6 and lesson updated.
- Turning measured: traction-limited yaw accel ~180 rad/s², full stick 44 rad/s, quarter stick 11 rad/s (linear stick). Asked the user how to tame it.

## 2026-10-10 05:30 Realistic fixture
**Asked:** turning fix → "Make the test robot realistic"; then fixture → "Replace simple_bot"; weapon → "Vertical disc, front"; layout → "Wedge front, axle rear (Recommended)".
**Did:**
- `make_fixture.py` rewritten: 14-part vertical spinner (base/rails/top/rear bar, battery, ESC, steel forks, inline drive motors, 63.5 mm wheels 30 mm behind centre on a 240 mm track, 150 mm × 10 mm AR500 disc at 4:1); 5.30 kg, CoM 58 mm ahead of the axle, yaw inertia 0.087 kg m² (was ~0.03). Full stick now settles at 24 rad/s (fork drag), half at 15.
- `rest.rs` counts contact by impulse (lesson `rapier-active-contacts-are-not-touching.md`); precession tests use the weapon's own axis; all importer/loader/CLI/server/physics tests updated to the new closed-form numbers.

## 2026-10-10 06:00 Restart dev server
**User:** restart
**Did:**
- Dev server (`tools/dev_server.py`, port 8417) had stopped with the previous session; restarted, serving the app, wasm and the new fixture.

## 2026-10-10 06:10 Lighting
**User:** remove the shadows and add overhead lights
**Did:**
- `web/src/arena_render.rs`: the shadow-mapped directional sun replaced by a 3 × 3 grid of shadowless point lights 3.5 m up (150,000 lm each, ~9,000 lx on the floor), each with an emissive panel; sidecar updated. Rebuilding the wasm.

## 2026-10-10 15:10 Too dark
**User:** its really dark
**Did:**
- `arena_render.rs`: added a shadowless downward directional "truss" light (6,000 lx) and global ambient fill (600 cd/m²), and raised the lamps to 600,000 lm; sidecar updated, rebuilding the wasm.

## 2026-10-10 15:40 Slidey
**User:** friction with the ground feels wrong, everything is slidey and ground contact feels wrong
**Did:**
- Probe (temporary, deleted) across rapier settings (Coulomb friction, no contact recycling, stiff contacts, 4 iterations): grip is Coulomb-correct in all of them: no creep under 15 N sideways, cornering lets go at ~0.6-0.9 g, braking 0.67 g (wheels carry ~75% of 0.9 g); only pivot wander improves (0.34 -> 0.08 m with stiff contacts + 4 iterations, ~1.7x cost).
- Suspect the visuals (no shadows since last change, untextured floor and wheels give no grounding or motion cues) and twitchy steering (10% turn at half speed asks 0.8 g); asked the user.

**Asked:** slidey when → "Stopping / letting go"; visual cues → "Floor texture (Recommended), Wheel markings (Recommended)"; physics → "Yes, stricter settings (Recommended)".
**Did:**
- `sim.rs` contact settings: Coulomb friction, no recycling, 200/400 Hz ζ 1 contacts, 4 iterations, joint warm-start (all tests pass). New `web/src/floor_texture.rs` (generated painted plywood, sheet seams, scuffs, mips) and 6-sector wheel markings in `robot_render.rs`; sidecars and doc §6 updated, wasm rebuilding.
- Stopping probe: centred sticks lock the wheels within 50-100 ms (short-circuit braking through 24:1 overpowers 0.9 grip) and it skids 1.48 m from 4.4 m/s at 0.67 g; asking the user what stopping should feel like.

**Asked:** stopping fix (options: more tyre grip / brake without locking / forks grip more / stops too slowly) → "the grip on the floor should either be sharpened steel cleats or vytaflex 40a polyurethane against wood"
**User (pasted):** FingerTech urethane sumo wheel spec: Shore A20 or A45 polyurethane, "Coefficient of Friction (against smooth unpainted MDF wood): μ=1.75".
**Did:**
- Fixture tyres are VytaFlex 40: density 1000 kg/m³, mu 1.4 (provisional: FingerTech's 1.75 on unpainted MDF, discounted for paint); prompt.py tyre guidance lists cast PU ~1.4 and says steel cleats have no number (ask). No sourced cleat figure found.
- Tests updated (masses 5.265 kg, straight run with the box moved away); stop from 4.45 m/s now 1.09 m in 0.5 s (was 1.48 m / 0.7 s), full turn 27.5 rad/s, slams still flat.

## 2026-10-10 16:20 Commit; tooth; camera
**User:** commit, driving feels more right now. could you add a big tooth to the vertical spinning disc?
and also zoom out the camera just a little bit
**Did:**
- Committed 8124f24. Found the wheel markings were never applied (a Python replace matched nothing after `cargo fmt`; the committed sidecar described them): applied now; lesson `scripted-edits-must-assert-their-match.md`.
- Fixture disc: r 70 mm with one tooth to r 90 (30 wide) and a 39.5 mm balance hole at r 40 on the tooth's side (first try put it opposite and doubled the imbalance); centre raised to z 95, battery shortened; 5.04 kg, tip speed 96 m/s. Importer/loader/CLI/server tests reworked (disc checked by balance and bounds; closed-form inertia moved to a wheel).
- Camera FOV 50 -> 58° (`FOV_DEG`), driver position unchanged. Web rebuilding.

## 2026-10-10 16:40 Box mass; commit
**User:** make the box mass match that of the bot, then commit
**Did:**
- `arena.rs`: `BOX_MASS_KG` removed; `add_target(world, x, y, mass_kg)`, and `Sim::new` passes the model's total mass (5.04 kg for the fixture). Test `the_box_weighs_what_the_robot_does`; sidecars and doc §6 updated; committed with the tooth, camera and wheel-marking work.

## 2026-10-10 17:00 Wall markings; camera back
**User:** add rulers and black/whtie squares on the wall to denote meters. also pull the camera back a little bit, when dr iving close to the camera it disappears from view
**Did:**
- New `web/src/wall_markings.rs`: 1 m black/white squares (one row) on the far and side walls, and red ruler ticks every 0.1 m (longer at 0.5 m and 1 m) on all four, counted from each wall's left corner; the near wall gets ticks only so it doesn't block the view.
- Camera `BEHIND_WALL_M` 1.0 -> 1.8 m: the bottom of the view now meets the floor outside the near wall (it was 0.24 m inside). Sidecars updated.
- Stopped two stale wasm builds and rebuilt once; `pkill -f` killed its own shell again, so I wrote lesson `pkill-f-matches-its-own-shell.md`.

## 2026-10-10 17:30 Wider bot; opponent copy
**User:** 1. make the simple-bot wider 2. replace the box with a copy of simple bot in orange
**User (interrupting):** make the bot twice as wide, 200mm to 400mm
**Did:**
- Fixture 400 mm wide, 440 mm track (base plate 3 mm, rear bar 6 mm to stay inside 12 lb): 5.28 kg, yaw inertia 0.14 kg m². The roll test now predicts from inertia instead of a fixed 0.05 rad.
- Box removed; `Sim` spawns an opponent copy at (1, 0) facing the robot, armed with sticks centred (`OPPONENT_DUTIES`: brakes, weapon off); motor stepping split into `crates/physics/src/drivetrain.rs` (one per robot). Web draws it orange. Tests: opponent faces the robot; full throttle shoves it 1.28 m in 3 s. Native physics 2.8x real time alone, 1.9x with the opponent.

## 2026-10-10 18:10 Flying on hits
**User:** 1.2x at minimum, wider robot feels great, braking is fine. the physics seem off, when i hit the opposing bot my bot tends to go flying, why is that?
**Did:**
- Probes (temporary): weapon off, the robot stays flat in every case. Weapon on, it launches even on open floor (yaw → gyroscopic roll → wheel load loop), under every solver setting, with or without the gyroscopic term or the tooth's lopsided inertia. Cause: the disc spun front edge DOWN (importer's arbitrary axis sign), so hits threw our robot instead of the opponent. Flipped: open floor flat (yaw 0.3 rad/s), hit throws the opponent 1.88 m and the robot recoils 0.32 m.
- New `crates/robot/src/spin.rs` + robot.toml `[[weapon]] spin` ("front-up"/"front-down", "ccw"/"cw"; defaults front-up / ccw), `LoadError::BadSpin`; fixture sets front-up; prompt documents and asks for it; doc §5 (also fixed its stale `channel = 3` example). Tests: `a_spinning_weapon_does_not_launch_the_robot_on_open_floor`, `a_hit_throws_the_opponent_up_not_the_robot`; all suites pass.

## 2026-10-10 18:20 Commit
**User:** commit
**Did:**
- Committed the wide fixture, the orange opponent copy, wall markings, camera move and weapon `spin`.
