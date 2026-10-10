# smoke.sh

## Function
End-to-end check of the import sandbox: one STEP goes through the broker's
exact docker command, as the service runs it (docs/drivesim.md §4).

## Interface
- `deploy/smoke.sh [runtime] [step]`, from the repo root inside nix-shell,
  or `make smoke-import RUNTIME=…`.
- `runtime` is `runsc` (default) or `runc`, for a Docker without gVisor
  registered. `step` defaults to the importer fixture.
- Prints `ok: N parts in T s` and exits 0, or fails with the sandbox's
  stderr.

## Implementation
- Builds and loads the image, then lays out a scratch robot dir as the
  server would: `robot.step` at 0644 and `bundle/` at 0777, because the
  sandbox runs as nobody.
- Calls `broker.run_import` itself, so the docker command is the broker's
  `import_argv`, not a copy that could drift.
- Checks that `parts.json` exists, that every mesh it names exists, and
  that a mesh is world-writable. That last one is the broker's `umask 0`:
  gVisor defaults to 022, which left the analyzer's files undeletable
  (its lesson, 2026-10-08).
- Scratch dirs go under `out/` (gitignored), not `/tmp`.
- **Results (2026-10-10, runc on WSL):** the fixture took 3.0 s, and
  Seraphs V1 (349 parts, 10 MB STEP) 41 s.

## Assertions
- [ ] Uses `broker.import_argv` through `run_import`; it never builds its
  own docker command.
