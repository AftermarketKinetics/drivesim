# server-image.nix

## Function
Builds `drivesim:latest`, the server container on accelerate. It holds the
FastAPI service, the robot checker, the motor library, the pages with
the browser sim, and the house robots' bundles.

## Interface
- `nix-build deploy/server-image.nix && docker load < result` (or
  `deploy/push.sh`).
- Serves on port 8000. Compose publishes no port; Caddy reaches it as
  `drivesim:8000`.
- Expects bind mounts at `/robots` (robot dirs, shared with the broker's
  host path) and `/data` (the SQLite database), and `ANTHROPIC_API_KEY`
  in its environment.
- Environment set here: `DRIVESIM_ROBOTS_ROOT=/robots`, `DRIVESIM_DB_PATH`,
  `DRIVESIM_IMPORT_MODE=broker`,
  `DRIVESIM_BROKER_URL=http://drivesim-broker:8711`, `DRIVESIM_CHECK_BIN`,
  `DRIVESIM_MOTORS_TOML`, `DRIVESIM_WEB_DIR=/app/web`,
  `DRIVESIM_HOUSE_ROOT=/app/house`.
- Runs as `1000:100`.

## Implementation
- **Adapted from the analyzer's `deploy/analyzer-image.nix`** (2026-10-10):
  a uv2nix env (`nix/server-env.nix`), `fakeNss`, `cacert`, and uvicorn
  with `--proxy-headers`.
- **The sim is the local build** (user, 2026-10-10). `server/web/sim/`
  comes from `make web-build`, which `push.sh` runs first. The build fails
  if the wasm is missing. Sidecar `.md` files stay out of `/app/web`.
- **House robots** (user, 2026-10-10; scaled versions 2026-10-11):
  `fixtures/simple_bot`, `simple_bot_1lb`, `simple_bot_3lb` and
  `simple_bot_30lb`, minus sidecars, are linked in under `/app/house/`.
  `house.py` copies them into `/robots` at every start and seeds them as
  public robots of their classes.
- **1000:100 owns the files it writes** in mint's `~/drivesim` bind mounts,
  as the operator. Sandbox-written files are world-writable (the broker's
  `umask 0`).
- **Verified locally** (2026-10-10), run as compose will with the broker
  on runc: 401 without the identity header; pages, wasm and radio model
  served `no-cache`; a fixture upload imported through the sandbox; one
  real Claude turn, recorded at 29,086 input-token equivalents.

## Assertions
- [ ] No port is published in compose; only Caddy reaches the service.
- [ ] `DRIVESIM_DEV_USER` is never set in this image.
- [ ] Paths match accelerate's compose volumes and `push.sh`'s dirs.
