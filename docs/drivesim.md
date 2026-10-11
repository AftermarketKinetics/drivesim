# drivesim: a driving simulator for NHRL robots

Status: **live 2026-10-11** at https://drivesim.combat-robot.systems
(§11 step 6). Built and tested: the STEP importer and its gVisor sandbox
(§4), the robot loader (§5), the physics with its validation tests (§6,
§9), the input mapping (§7), the server with its role conversation and
daily budget, and the browser app. Decisions in §2 are the user's;
anything else marked *proposal* is open to review.

## 1. Goal

Practise driving a combat robot that does not exist yet, or that cannot be
tested at home without a cage. Anyone signed in to **accelerate** (the
user's self-hosted site, behind authentik) uploads a robot as a **STEP
assembly**, answers a few questions about it, and drives it **in the
browser** with their real radio (an EdgeTX transmitter in USB joystick
mode). The robot behaves as a 3D rigid body: weight shifts, tips, flips
and the weapon's **gyroscopic precession** appear because the physics
integrates them, not because they are scripted.

The first robot is Momentum (`~/code/momentum`), an NHRL 12 lb horizontal
spinner on a tail. The tool is general: Momentum is a robot like any
other, plus one optional hook (its firmware core in the loop, §6).

**v1:** upload a STEP, set its roles, drive it around a walled arena with
a full-speed weapon, and hit an undriven opponent: a copy of the same
robot, drawn orange, on braked wheels (user, 2026-10-10; it replaced a
free box). The sim page also offers a box and an AI-driven copy (§2),
and the AI can drive any public robot instead of the copy (§2).

**Not in v1:** a second, different robot driven by a person; damage and part failure;
tagging parts by clicking in the 3D view; telemetry replay; arena
hazards; a native desktop build.

## 2. Decisions (user, 2026-10-09)

- **Own repository**; Momentum consumes it. Momentum's STEP and robot
  file live in the Momentum repo.
- **Runs in the browser, hosted on accelerate, in v1.** A separate app
  from the Combat Robot Analyzer, behind the same authentik login.
  Chrome / Edge are the supported browsers. (This replaced the earlier
  native Windows .exe plan the same day.)
- **Robots come in as STEP uploads, imported on the server.** The
  importer runs in a **gVisor sandbox started by drivesim's own broker**,
  because a STEP upload is untrusted input to a large C++ parser.
- **The server copies and adapts the Combat Robot Analyzer's** auth, store,
  event bus, agent loop and broker pattern, rather than sharing a library
  or starting fresh. The role conversation runs on `claude-sonnet-5-5`,
  with the analyzer's API key on accelerate. Robots are visible to their
  owner only in v1. (2026-10-10: plus admins, named in
  `DRIVESIM_ADMINS`, who may read and drive every robot from
  `/admin-dashboard`; replying and deleting stay with the owner.)
- **Roles come from a Claude conversation.** A STEP does not say which
  part is a wheel or the weapon, or what anything is made of; Claude asks
  the user and writes the robot file. The user first asked for this as a
  Claude Code skill; with server-side import it runs in the web page
  through the Anthropic API.
- **Engine: Rust + Bevy + rapier3d.** Physics is a Bevy-free crate; Bevy
  only draws. Rapier 0.36 integrates the gyroscopic term itself (on by
  default; an explicit `−ω × Iω` step with the momentum norm held
  constant, as Jolt does).
- **Weapon: full model only.** A rigid body on a motor-driven revolute
  joint, spinning at true RPM.
- **Wheels are real bodies on joints**, gripping through rapier contact
  friction.
- **Drivetrain in the robot file, with a motor library.**
- **Momentum goes through the same importer** as any other robot.
- **Development is local first:** the WASM build is served from WSL on
  localhost with cross-origin isolation; nothing is deployed until the
  last step (§11).
- **Opponent: copy, box or AI** (user, 2026-10-10), chosen with three
  buttons on the sim page, each switch restarting the round. The AI
  drives the orange copy: it waits a 3-2-1 countdown, then chases the
  robot with its weapon always at full, backs out when wedged, and
  drives inverted when flipped. Whenever its weapon is under 70% of
  speed (the start, or after a hit) it backs off and waits for 90%
  (user, 2026-10-11). No difficulty setting, no pathfinding,
  no aiming of the weapon side. This brought an AI opponent into v1.
  AI rounds are **3-minute bouts** (user, 2026-10-11): the clock starts
  at FIGHT, and at 0:00 both robots are disarmed, "TIME" shows and a
  Rematch button rebuilds the round. The copy and the box stay untimed.
  **AI vs AI** (user, 2026-10-11) is a fourth button: an AI drives the
  robot too, in place of the radio, so a robot can be tried with no
  radio set up. The home page's "How it works" lists the four modes.
- **Public robots as AI opponents** (user, 2026-10-10). An owner can tag
  a drivable robot "publicly accessible" with its weight class (3, 12 or
  30 lb, the owner's pick, not derived from mass). Every signed-in user's
  sim page then lists the public robots in a dropdown shown while AI is
  chosen, grouped by class with the driven robot's class first. The
  first entry is the robot's own copy. Publishing shares the robot's
  bundle (3D model and robot file), never its conversation. The first
  public set is simple_bot (12 lb) with its 1, 3 and 30 lb versions
  (user, 2026-10-11: scaled geometry with each class's own motors;
  1 lb joined the classes for them), seeded at every start from
  `fixtures/` as a database robot owned by `drivesim-house`, and Seraphs
  V1, which its owner ticks public. Any class can face any class; the
  arena stays the 12 lb cage. This brought "a second, different robot as
  opponent" into v1, for the AI only.
- **Lossy impacts** (user, 2026-10-11): collisions lose energy, as real
  ones do to deformation, heat and sound. Every impact except with the
  floor keeps half its velocity change (§6). Controls do not invert
  when a robot is upside down, as on a real robot without an invert
  switch; a radio mix on a switch does it, as at an event.

## 3. Pipeline

```
browser                                    accelerate
───────                                    ──────────
upload robot.step ──────────────────────>  drivesim-server (FastAPI)
                                              │ asks the broker
                                              ▼
                                           gVisor sandbox: importer
                                              │ parts.json + meshes
                                              ▼
role chat in the page  <────────────────>  Claude (Anthropic API)
                                              │ writes robot.toml
                                              ▼
                                           robot bundle, per user
WASM app (Bevy) <── fetch bundle ─────────────┘
  ├─ drivesim-robot   (bundle -> model)
  ├─ drivesim-physics (rapier3d)
  ├─ drivesim-input   (Gamepad API -> channels)
  └─ control: mixer, or a firmware core
```

## 4. Import

**The importer** (`tools/import/`, Python with `pythonocc-core`) turns a
STEP into a parts directory. Built and tested 2026-10-09. Geometry only,
no roles:

- `parts.json`: one record per solid. Instance and product names, tree
  path, colour, bounding box, and its exact B-rep volume, centre of
  volume and inertia tensor **at unit density**, in SI units in the
  STEP's own frame.
- `rotation_axis`: the line the solid's revolved faces turn about, with
  the outer radius and width measured from the mesh, the share of the
  surface that is curved about it and flat square to it, and the
  distance from the part's centre to the line. That last number is what
  separates a rotor (wheel, blade: 0.0 mm on Momentum's parts) from a
  plate with a bore (Momentum's floor: 126 mm).
- One binary STL per solid, wound outward, in metres.

It follows the user's `cad-step` plugin (`~/code/cad-step-parser-skills`)
for the tree walk, copied rather than imported so the two can change
separately.

**On the server**, each upload becomes a job: the broker starts a
sandbox from the importer image with no network, the STEP mounted
read-only and an output directory writable, with CPU, memory and time
limits. The analyzer's `cra-broker` is the pattern; drivesim runs its
own.

**The role conversation** (`server/drivesim_server/`, built 2026-10-09).
Claude's tools are `parts_summary`, `list_motors`, `show_robot_toml` and
`write_robot_toml`; it reads the parts as text, not the meshes. It proposes
roles from the geometry (a wheel is a rotor whose radius reaches the floor,
normally in a left/right pair; the weapon is the rotor with the most mass
far from its axis; the rest is chassis) and materials from part names, and
asks the user about anything it is unsure of, in the page, with the
evidence. Every write runs `drivesim-check` (`crates/cli`), the sim's own
loader and physics, which reports mass, CoM, inertia, each rotor's no-load
speed and how the robot rests (tilt, which bodies touch the floor, after a
small nudge so a robot balanced on its axle falls as a real one would). A
wrong role shows up as a wrong number before anyone drives.

## 5. Robot file

`robot.toml`, stored with the parts. A draft of the shape, not yet a
schema:

```toml
[robot]
name = "Momentum"
up = "+z"                    # the STEP's up and forward axes
forward = "+x"
total_mass_kg = 5.44         # optional: scale every part to the weighed total

[materials]                  # glob on instance/product names -> material
default = "pa12"
"Blade*" = "ar500"
"Wheel*" = "tpu"

[mass_overrides]             # optional per-part masses, kg
"floor" = 0.164

[[wheel]]
parts = ["Wheel-L*"]         # solids that turn together
motor = "<library id>"
ratio = 6.0
mu = 0.8

[[weapon]]
parts = ["Blade*", "Hub*"]
motor = "<library id>"
ratio = 1.0
spin = "front-up"            # cross axis: front-up | front-down;
                             # upright axis: ccw | cw seen from above

[control]
kind = "mixer"               # or "momentum-core" (§6)
throttle = 2
turn = 1
```

- **A weapon's `spin`** picks its direction. The importer's axis sign is
  arbitrary, and the fixture's vertical disc first spun front edge down.
  That made the robot fly on every hit, and under full throttle it
  launched itself (user report and probe, 2026-10-10). Defaults:
  front-up on a cross axis, ccw from above on an upright one.
- A part's mass is its volume times its material's density, unless
  overridden; `total_mass_kg` then scales every part together, keeping
  the CoM where the CAD puts it.
- **The motor library** (`data/motors.toml`): kind (brushless or
  brushed), kv, winding resistance, current limit, pole pairs, rotor
  inertia for common combat motors, 1 lb to 30 lb (43 motors, filled
  from NHRL Discord and vendor pages, 2026-10-10). Both kinds use the
  same DC motor model; a brushed gearmotor's kv is the bare motor's, its
  gearbox going in `ratio`. Every number can be overridden.
- **Momentum's hook:** its printed parts must not use solid volume ×
  density (its repo rule: sliced masses only), so its robot file fills
  `mass_overrides` from `make masses`, generated on the Momentum side.

## 6. Physics

**Engine:** rapier3d 0.36 in `crates/physics`, with no Bevy, stepped by
its own fixed-tick loop.

**Bodies:**

- **Chassis:** one body for every part without another role. Collider:
  the convex hulls of the body's 12 biggest parts by volume. Small parts
  (fasteners, magnets, internals) collide not at all but keep their mass.
  The first scheme (2026-10-09) gave every part a hull, or a V-HACD
  decomposition where the hull overfilled it. On the first real robot
  (Seraphs V1, 349 parts, 2026-10-10) that made 695 colliders across it
  and its opponent copy, took 46 s to build and ran at 0.01× real time;
  twelve hulls per body build in 0.1 s and run at 3.6×. Mass, CoM and inertia are set from the importer's
  exact numbers, not the colliders.
- **Wheels:** a body per wheel group on a revolute joint, axis from the
  importer, cylinder collider at the measured radius and width. Torque
  from the DC motor model carried over from Momentum's 2D sim:
  `I = clamp((d·V − ω·ratio/kv)/R, ±i_max)`, torque `I/kv · ratio · η`.
- **Weapon:** a body on a revolute joint, same motor model, true RPM,
  gyroscopic forces on, CCD on. It collides only through the parts
  reaching at least half its outer radius (user, 2026-10-10,
  `weapon_reach.rs`): spindles and pulleys inside the robot stay in its
  mass but cost no collision time.
- **Motors act as torques,** not rapier joint motors: each physics step
  the motor model's torque goes onto the rotor about its axis and the
  opposite onto the chassis, so reaction torque is real. (Rapier's
  velocity motors never moved the spike robot.) Each rotor body also
  carries its motor's rotor inertia reflected through the gearing
  (`J · ratio²`), which a real gearmotor has and which keeps the explicit
  torque stable at the physics step. Wheels have rapier's gyroscopic term off,
  because the lumped inertia overstates their angular momentum about
  `ratio` times. The lump would also put ~`ratio` times too much
  spin-up reaction on the chassis (the fixture flipped on every hard
  reversal), so each step adds the gearbox coupling torques that turn it
  into a real gearmotor (`gear_coupling.rs`).
- **Contact settings** (user, 2026-10-10): rapier's exact Coulomb
  friction per contact point, no contact recycling, stiff critically
  damped floor contacts (200/400 Hz) and 4 solver iterations, for about
  1.7× the physics cost. The defaults (a single friction cone plus a
  twist constraint per patch, recycled points, soft 30 Hz contacts
  tuned for piles) gave the same grip but more wander when spinning in
  place.
- **Impact loss** (user, 2026-10-11, `impact_loss.rs`): restitution
  is 0 everywhere, but a rigid contact still hands the struck body all
  of a weapon tooth's speed. After each step, a robot (or the box) whose
  contacts with anything but the floor exceed 5 g keeps half the velocity
  change that step gave it. Rotor spin is the exception: the weapon
  still loses all the speed the hit took. Measured on the fixture: a hit
  throws the opponent at 3.7 m/s, not 6.7; a weapon striking the wall
  recoils at 0.5 m/s, not 5.8.
- **Impact marks** (user, 2026-10-11, `impacts.rs`, web
  `impact_marks.rs`): a hit that changes a robot's speed by 2 m/s or more
  in one step (weapon hits, not shoves) leaves a mark where it landed. It
  glows orange to red and grows with the hit's strength, and keeps
  glowing for the round (user, 2026-10-11: no fading; camera bloom so it
  halos). Wheels lost their stripes so they don't clash with the marks.
- **Arena:** NHRL's 12 lb cage: 16' × 16' usable, a floor of plywood
  painted with dead-flat paint (not steel), polycarbonate walls on a timber
  skirt, a roof (NHRL wiki, read 2026-10-09); the roof is simulated too, on
  the walls' tops, so a hit can't throw a robot out (user, 2026-10-10). **Opponent:** an undriven copy of the robot, drawn orange,
  armed with sticks centred so it brakes (user, 2026-10-10; it replaced
  a free target box); or the box; or the copy driven by
  `ai_driver.rs` after a countdown of sim time (§2). The AI can drive a
  rival robot instead of the copy (`Sim::versus`): spawned at the copy's
  spot, it brings its own model, so its own masses, motors and gearing.

**Timestep.** The physics step is sized from the fastest rotor, not the
control rate. Measured 2026-10-09 (timing spike, lesson
`rapier-fast-weapon-needs-small-dt.md`): a 10,000 RPM weapon on a rapier
revolute joint throws the robot at a 1 ms step whatever the substep
count, and is stable from 2 kHz (30° per step) up. It ran at 4 kHz
(15° per step) until **2 kHz physics** (user, 2026-10-10, for speed:
Obliterator 6 v72 against its AI copy ran at 0.7× real time in the
browser, 1.31× → 2.4× natively), with the control tick (1 kHz for
Momentum's core) every second step. Weapon CCD stays on: without it
hits stopped throwing the opponent. In WASM on one thread the whole spike scene ran
at 6.6× real time at 4 kHz (Node 24 / V8) and 5.4–6.1× in the user's
Chrome, so threads (and the cross-origin isolation they need) are not
needed for v1.

**Sensors.** A simulated IMU reads the chassis body's real angular
velocity and specific force.

**Control:** `mixer` (arcade or tank mixing from channel numbers; any
robot) or `momentum-core` (Momentum's `no_std` firmware core, as a Cargo
dependency, fed by the simulated IMU and wheel speeds, so practice
includes the real drive assists).

## 7. Input

The radio runs EdgeTX in USB joystick mode (Classic). On Windows natively
it reads as a generic HID controller whose channel N is raw axis N−1,
through `gilrs-core` (not `gilrs`, whose guessed layout flips and
rescales axes; Momentum's Pocket, 2026-10-09). In the browser
the page reads the **Gamepad API** directly. **Verified in Chrome
2026-10-09** with `tools/padtest/`: the Pocket appears as
`"Radiomaster Pocket Joystick (Vendor: 1209 Product: 4f54)"` with
`mapping=""` (no browser remapping), 8 axes in channel order with the
radio's own sign, and 24 buttons (unused). Vendor 0x1209 / product
0x4F54 is EdgeTX's USB joystick ID, so the sim finds the radio by ID, not
name. `crates/input` holds
the conversion and is host-tested. No radio means no frames, which a
controller treats as signal loss.

## 8. Layout and build

```
Cargo.toml              workspace: robot, physics, input (host-tested)
crates/robot/           robot.toml schema, motor library, bundle loader
crates/physics/         rapier3d world, bodies, sensors, controllers
crates/input/           radio axes -> channels
crates/cli/             drivesim-check: a bundle -> JSON report
web/                    Bevy app, its own workspace, wasm32 only
server/                 FastAPI: uploads, jobs, role chat, bundles, static app
broker/                 starts gVisor import sandboxes; holds the Docker socket
tools/import/           STEP importer (Python, pythonocc)
nix/                    the shared nixpkgs pin and the importer's Python
deploy/                 nix-built images (import sandbox), smoke test
data/motors.toml
shell.nix               pinned nixpkgs: Rust (+ wasm32 std), lld, wasm-bindgen,
                        binaryen, Python with pythonocc, uv, docker CLI
                        (server/ Python packages are uv's: pyproject + uv.lock)
```

- **Library crates take bytes, never paths**, and use no native-only
  dependencies, so the same code is host-tested and runs in WASM.
- The web app is its own workspace, so host tests never build Bevy.
- The nixpkgs pin's host `rustc` already ships the `wasm32-unknown-unknown`
  standard library; `wasm-bindgen-cli` (0.2.127) and `binaryen` come from
  the same pin, and the `wasm-bindgen` crate is pinned to match exactly.
- Images are built by nix and loaded on accelerate, as the analyzer's are.
- Sidecar docs, lessons, session logs and nix-first tooling follow the
  user's global conventions.

## 9. Validation

Host tests, run before any driving is trusted:

- **Precession magnitude** (passing): gravity off, weapon at 1,000
  rad/s, 0.5 N m across the spin: the spin axis precesses at τ/L within
  10% and barely tips; without spin the same torque just tips it.
- **Momentum conservation** (passing): nothing driven, chassis tumbling,
  weapon spinning: total angular momentum within 1% over 1 s.
- **Substeps:** the precession result holds at the chosen count and
  drifts at fewer.
- **Wheels** (passing): 10% throttle reaches 97.6% of the motor
  model's no-load speed, straight; turning goes the right way; disarmed
  stays put; the weapon spins up to no-load with the robot upright; the
  robot shoves its braked opponent copy.
- **Import** (passing): the fixture STEP imports to known masses, axes
  and names; the loader turns it into analytic masses and inertia.
- **Momentum:** with `momentum-core`, a strike run end to end flicks the
  tail.
- **WASM parity:** the physics crate's tests also run under
  `wasm32-unknown-unknown`, so the browser build integrates the same way.

## 10. Open questions

- **WASM speed with real robots.** Seraphs V1 runs at 3.6× real time
  natively with twelve hulls per body; the browser is slower. A robot
  whose outline depends on small parts (a thin wedge tip, a skid) may
  need those parts to get a shape; no robot file setting exists for that
  yet.
- **Wheel friction feel.** Rapier's contact friction is Coulomb, with no
  slip curve; driving may feel too grippy. A contact-modification hook
  could shape friction by slip if needed.
- **Small fast wheels on contacts** may jitter at the substep count the
  weapon needs.
- **Weapon hits are approximate.** CCD handles linear motion well and
  fast rotation poorly. Fine for driving feel and hit timing, not for
  damage numbers (Momentum's OpenRadioss workflow covers those).
- **Floor grip:** tyre `mu` values are for painted plywood; there are no
  measured numbers yet.
- **Momentum's assembly STEP:** a whole-robot export from Fusion does not
  exist yet.
- **`momentum-core` in the server build:** a path dependency works
  locally; the nix image build needs it as a git dependency or a vendored
  copy.
- **API cost and limits** for the role conversation: one conversation
  per robot, but per-user limits are still to set.
- **Momentum's superseded 2D sim:** its uncommitted `drivesim/physics`
  crate (rapier2d, 21 tests) is dropped from Momentum; the motor model and
  Pocket findings carry over here.

## 11. Build order

1. This doc; repo conventions (CLAUDE.md, sidecars, Makefile).
2. **Browser spike, local:** a web pad test (the radio through the
   Gamepad API in Chrome), and rapier3d in WASM with a full-RPM weapon
   body, timed.
3. `crates/robot`, then `crates/physics` with the §9 tests.
4. `server/` and `broker/`: upload, sandboxed import, role conversation,
   per-user storage. **Done 2026-10-09** (local: 38 server tests with a
   scripted model, 5 broker tests without Docker, and one live Claude
   conversation that wrote a checked robot.toml). **Import sandbox checked
   2026-10-10:**
   - `deploy/import-image.nix` builds `drivesim-import:latest`;
   - the importer under gVisor (`runsc do`, rootless) writes output
     byte-identical to the fixture bundle;
   - the broker's exact docker command imports the fixture (3 s) and
     Seraphs V1 (349 parts, 41 s) on runc (`deploy/smoke.sh`);
   - the server in broker mode imports through the real broker process.

   accelerate already runs the analyzer's sandboxes on runsc, so Docker
   plus runsc with this image is first exercised there.
5. `web/`: load a bundle, drive, hit the opponent copy. **Renders in the user's
   Chrome 2026-10-10** (the fixture bundle from `make web-dev`); the HUD
   shows physics headroom (× real time) so build changes can be checked
   in the browser.
6. Deploy on accelerate (asks the user first: it changes the live site).
   Plan (user, 2026-10-10), modelled on the analyzer's v2 deploy:
   - **Where:** `drivesim.combat-robot.systems`; anonymous visitors get a
     public landing page (what drivesim is, the radio guide, sign in);
     any authentik user may use it, each under the daily budget.
   - **Phase A, local:**
     - images: `deploy/server-image.nix` (uv2nix service, `drivesim-check`,
       motor library, web pages plus the locally built sim, which
       `push.sh` rebuilds every time), `deploy/broker-image.nix`, and the
       import image;
     - `deploy/push.sh`: streams the images into `docker load` on mint and
       creates `~/drivesim/robots` (0777) and `~/drivesim/data` (0700);
     - the analyzer's caching fixes (no-cache HTML, content-hash ETags)
       and response compression;
     - the production images verified locally, with one real Claude turn;
     - accelerate edits: compose services `drivesim` and
       `drivesim-broker` on an internal network, a Caddy block copied from
       the analyzer's, and the tunnel record.
   - **Phase B, live, each command approved:**
     - push the images;
     - an authentik forward-auth provider and application, added to the
       embedded outpost;
     - compose and Caddyfile to mint, then reload Caddy (wiki and analyzer
       still 200);
     - the user: DNS, the tunnel ingress in mint's `configuration.nix`,
       then `nixos-rebuild` (runsc already registered; API key from the
       analyzer's `.env`);
     - start the services, then `deploy/smoke.sh runsc` on mint and a
       login, upload and drive;
     - commit both repos.
