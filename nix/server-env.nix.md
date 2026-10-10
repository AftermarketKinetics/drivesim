# server-env.nix

## Function
drivesim-server's Python environment, built by uv2nix from
`server/uv.lock`, for the server image.

## Interface
- `import ./nix/server-env.nix { inherit pkgs; }` gives a virtualenv with
  `drivesim_server` and its locked dependencies, including `uvicorn`.

## Implementation
- Adapted from the analyzer's `nix/analyzer-env.nix`, with the same
  pinned pyproject.nix, uv2nix and build-system-pkgs revisions.
- `onlySources` copies only `pyproject.toml` and `drivesim_server/**/*.py`.
  `server/` also holds `.venv/`, `var/` (robots, the database) and
  `tests/`, none of which may reach the store or the image.

## Assertions
- [ ] Built from `uv.lock`, so production matches the tested versions.
