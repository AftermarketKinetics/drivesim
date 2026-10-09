# Makefile

## Function

The repo's entry points, run inside `nix-shell`.

## Interface

- `make test`: every test suite: the importer's and the crates'.
- `make test-rust`: `cargo test`, `clippy -D warnings` and `fmt --check`
  over the workspace.
- `make test-import`: `tools/import/tests` under `unittest`.
- `make web-dev`: serves the repo on http://localhost:8417 with
  cross-origin isolation (`tools/dev_server.py`).

## Implementation

Targets call tools from `shell.nix` only; nothing is installed by a
target.

## Assertions

- `make test` runs every suite that exists, so one command gates a commit.
