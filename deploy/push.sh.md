# push.sh

## Function
Ships drivesim's images to mint: builds the import, broker and server
images with nix, and streams each into
`docker load` on mint.

## Interface
- `deploy/push.sh [--dry-run]` from the repo root inside nix-shell;
  `MINT_HOST` overrides the ssh target (default `mint`).
- Effects on mint:
  - `drivesim-import`, `drivesim-broker` and `drivesim` images loaded;
  - `~/drivesim/robots` (0777) and `~/drivesim/data` (0700) exist.
- `--dry-run` builds and prints sizes only.

## Implementation
- **Copied from the analyzer's `deploy/push.sh`:** streams
  `ssh mint docker load < tarball`, so mint never stores the tarball, and
  starts nothing. Services are brought up from accelerate's compose
  (`drivesim-broker`, `drivesim`).
- **No `make web-build` step** (since 2026-10-19): the server image builds
  the sim in nix (`nix/sim.nix`), so a stale local build can't ship.
- **Being retired:** on clover, accelerate takes this repo as a flake input
  and arion loads the images; this script only serves mint until cutover.
- `~/drivesim/robots` is world-writable because each import sandbox runs as
  uid 65534; `~/drivesim/data`, the database, is private.

## Assertions
- [ ] Never restarts or recreates a running container.
- [ ] Paths match `DRIVESIM_ROBOTS_HOST_ROOT` and the volume mounts in
  accelerate's compose file.
