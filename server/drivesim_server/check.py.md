# check.py

## Function
Runs `drivesim-check` (crates/cli) on a robot's bundle. The role
conversation checks its `robot.toml` with the same Rust loader and physics
that will drive the robot.

## Interface
`async check_bundle(settings, bundle) -> dict`: the checker's JSON report
(see `crates/cli/src/report.rs.md`), or `{"ok": False, "error"}`.

## Implementation
- `TIMEOUT_S` is 120. A timeout, a crash, or output that isn't JSON each
  become an `ok: false` report carrying the stderr tail, so the agent reads
  the failure instead of the tool raising.
- The binary is `settings.check_bin`, built by `cargo build --release -p
  drivesim-cli` (`make test-server` and `make server-dev` build it first).

## Assertions
- [ ] Always returns a dict with `ok`; never raises on a checker failure.
