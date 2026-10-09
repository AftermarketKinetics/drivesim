# drivesim: a driving simulator for NHRL robots

Status: **design, 2026-10-09.** Built: the STEP importer (§4), the
robot loader (§5), the input mapping (§7) and the physics with its
validation tests (§6, §9). Planned: the server, the web app, deployment. Decisions in §2 are the user's; anything else marked
*proposal* is open to review.

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
a full-speed weapon, and hit a free box. No opponent.

**Not in v1:** a second robot or AI opponent; damage and part failure;
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

**The role conversation.** Claude gets `parts.json` (not the meshes) and
a tool to write `robot.toml`. It proposes roles from the geometry (a wheel
is a rotor whose radius reaches the floor, normally in a left/right
pair; the weapon is the rotor with the largest inertia about its own
axis; the rest is chassis), and materials from part names; it asks the
user about anything it is unsure of, in the page, with the evidence; it
then runs the same loader the sim uses and shows total mass, CoM, inertia
and the resting contact points, so a wrong role shows up as a wrong
number before anyone drives.

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
channel = 3

[control]
kind = "mixer"               # or "momentum-core" (§6)
throttle = 2
turn = 1
```

- A part's mass is its volume times its material's density, unless
  overridden; `total_mass_kg` then scales every part together, keeping
  the CoM where the CAD puts it.
- **The motor library** (`data/motors.toml`): kv, winding resistance,
  current limit, pole pairs, rotor inertia for common combat motors.
  Every field can be overridden by number.
- **Momentum's hook:** its printed parts must not use solid volume ×
  density (its repo rule: sliced masses only), so its robot file fills
  `mass_overrides` from `make masses`, generated on the Momentum side.

## 6. Physics

**Engine:** rapier3d 0.36 in `crates/physics`, with no Bevy, stepped by
its own fixed-tick loop.

**Bodies:**

- **Chassis:** one body for every part without another role. Collider:
  one convex hull per part, or parry's V-HACD decomposition for a part
  its hull overfills by more than 1.5× (brackets, rings); *proposal,
  2026-10-09*: cheaper than decomposing everything, and most robot parts
  are near-convex. Mass, CoM and inertia are set from the importer's
  exact numbers, not the colliders.
- **Wheels:** a body per wheel group on a revolute joint, axis from the
  importer, cylinder collider at the measured radius and width. Torque
  from the DC motor model carried over from Momentum's 2D sim:
  `I = clamp((d·V − ω·ratio/kv)/R, ±i_max)`, torque `I/kv · ratio · η`.
- **Weapon:** a body on a revolute joint, same motor model, true RPM,
  gyroscopic forces on, CCD on.
- **Motors act as torques,** not rapier joint motors: each physics step
  the motor model's torque goes onto the rotor about its axis and the
  opposite onto the chassis, so reaction torque is real. (Rapier's
  velocity motors never moved the spike robot.)
- **Arena:** floor and four walls; NHRL cage size TBD (§10). **Target:** a
  free box, 5.44 kg, 300 × 300 × 150 mm.

**Timestep.** The physics step is sized from the fastest rotor, not the
control rate. Measured 2026-10-09 (timing spike, lesson
`rapier-fast-weapon-needs-small-dt.md`): a 10,000 RPM weapon on a rapier
revolute joint throws the robot at a 1 ms step whatever the substep
count, and is stable from 2 kHz (30° per step) up. **Proposal: 4 kHz
physics** (15° per step) with the control tick (1 kHz for Momentum's
core) every fourth step. In WASM on one thread the whole spike scene ran
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
web/                    Bevy app, its own workspace, wasm32 only
server/                 FastAPI: uploads, jobs, role chat, bundles, static app
broker/                 starts gVisor import sandboxes; holds the Docker socket
tools/import/           STEP importer (Python, pythonocc)
deploy/                 nix-built images, push to accelerate
data/motors.toml
shell.nix               pinned nixpkgs: Rust (+ wasm32 std), wasm-bindgen,
                        binaryen, Python with pythonocc and FastAPI
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
  robot pushes the box.
- **Import** (passing): the fixture STEP imports to known masses, axes
  and names; the loader turns it into analytic masses and inertia.
- **Momentum:** with `momentum-core`, a strike run end to end flicks the
  tail.
- **WASM parity:** the physics crate's tests also run under
  `wasm32-unknown-unknown`, so the browser build integrates the same way.

## 10. Open questions

- **WASM speed with real colliders.** The spike's 6.6× margin is with
  boxes and cylinders; V-HACD compounds of a real robot cost more. To be
  re-measured with Momentum's imported geometry.
- **Wheel friction feel.** Rapier's contact friction is Coulomb, with no
  slip curve; driving may feel too grippy. A contact-modification hook
  could shape friction by slip if needed.
- **Small fast wheels on contacts** may jitter at the substep count the
  weapon needs.
- **Weapon hits are approximate.** CCD handles linear motion well and
  fast rotation poorly. Fine for driving feel and hit timing, not for
  damage numbers (Momentum's OpenRadioss workflow covers those).
- **Arena dimensions:** NHRL's 12 lb cage size, floor and wall materials.
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
   per-user storage.
5. `web/`: load a bundle, drive, hit the box.
6. Deploy on accelerate (asks the user first: it changes the live site).
