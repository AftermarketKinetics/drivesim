# drivesim

Drive a combat robot before it exists. Upload the robot's CAD as a STEP
assembly, answer a few questions about it, and drive it in the browser in
NHRL's 16 ft cage, from the driver's box, with your own EdgeTX radio.

Live at **https://drivesim.combat-robot.systems** (invite-only; accounts come
from [wiki.combat-robot.systems](https://wiki.combat-robot.systems/)).

## How it works

1. **Import.** The STEP is imported in a gVisor sandbox (`tools/import`,
   pythonocc): parts, exact masses and inertias, rotation axes, meshes.
2. **Set up.** A Claude conversation works out with you which parts are
   wheels and weapon, materials, motors and gear ratios, and writes the
   robot file (`robot.toml`). The checker validates every version.
3. **Drive.** The browser app (Bevy + rapier3d, compiled to WebAssembly)
   simulates the robot as 3D rigid bodies at 2 kHz: DC motors through their
   gearboxes, tyre grip, and the weapon's gyroscopic effect. Your radio
   connects as a USB joystick. An orange copy of your robot is the target.

The design, with every decision and measurement, is
[`docs/drivesim.md`](docs/drivesim.md).

## Layout

```
crates/robot     robot.toml schema, motor library, bundle loader
crates/physics   rapier3d world: bodies, motors, gearbox coupling, contact
crates/input     radio axes -> channels
crates/cli       drivesim-check: a robot bundle -> JSON report
web/             the browser app (its own Cargo workspace, wasm32)
server/          FastAPI: uploads, the setup conversation, budgets, pages
broker/          starts one import sandbox per upload (holds the Docker socket)
tools/import/    the STEP importer
deploy/          nix-built images and push.sh
nix/             the shared nixpkgs pin and Python environments
data/            the motor library
fixtures/        the test robot (a 12 lb vertical spinner)
```

## Develop

Everything runs inside the nix shell, which provides the whole toolchain
(Rust with wasm32, wasm-bindgen, Python with OpenCASCADE, uv, gVisor):

```sh
nix-shell --run 'make test'         # importer, crates (tests, clippy, fmt), server, broker
nix-shell --run 'make web-build'    # the browser app -> server/web/sim/ (~13 min)
nix-shell --run 'make server-dev'   # the server on localhost:8420 (dev user, local import)
nix-shell --run 'make web-dev'      # the repo on localhost:8417: the sim with the test robot at
                                    # /server/web/sim/?bundle=/fixtures/simple_bot/&motors=/data/motors.toml
```

`make server-dev` uses Claude through your `ANTHROPIC_API_KEY` or an
`ant auth login`. The sim needs Chrome or Edge (WebGPU).

## Deploy

`deploy/push.sh` rebuilds the browser app, builds the server, broker and
import images with nix, and loads them into Docker on mint. It starts
nothing; the services run from accelerate's `docker-compose.yml`
(`drivesim`, `drivesim-broker`) behind Caddy and authentik.

## Conventions

Every code file has a sidecar doc beside it (`<file>.md`), lessons learned
are in `.claude/lessons/`, and dependencies come from nix first. See
`CLAUDE.md`.
