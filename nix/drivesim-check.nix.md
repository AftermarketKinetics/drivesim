# drivesim-check.nix

## Function
Builds `drivesim-check` (crates/cli), the robot checker the server runs
on each robot file, for the server image.

## Interface
- `import ./nix/drivesim-check.nix { inherit pkgs; }` gives
  `bin/drivesim-check`.

## Implementation
- `rustPlatform.buildRustPackage` on the root workspace with its
  `Cargo.lock` and `-p drivesim-cli`.
- Its source fileset is the workspace manifests and `crates/` (sidecars
  excluded); `target/`, `web/` and the fixtures stay out.
- Tests are off here; they need the fixtures and run in `make test`.
- Checked 2026-10-10: the nix-built binary reports the fixture at
  5.282 kg, as the cargo build does.

## Assertions
- [ ] The same `Cargo.lock` as development.
