# import-image.nix

## Function
Builds `drivesim-import:latest`, the image every STEP upload is imported in
(docs/drivesim.md §4). The broker starts one container per upload, with
gVisor `runsc`, no network and a read-only root; it runs the importer once
and exits.

## Interface
- `nix-build deploy/import-image.nix && docker load < result`, or
  `make import-image`.
- `/opt/import/*.py`: the importer modules (`step_import.py` and what it
  imports), the path `broker/broker.py` `import_argv` runs.
- `/work` is the working directory, where the broker mounts `robot.step`
  read-only and `bundle/` writable. `/tmp` exists for the broker's tmpfs.
- `sh` (dash) and `timeout` (coreutils), for the broker's fixed
  `sh -c 'umask 0 && exec timeout -s KILL …'` command.
- Runs as `65534:65534` (nobody). `PATH=/bin`, `HOME=/tmp`.

## Implementation
- **Adapted from the analyzer's `sandbox/image.nix`** (2026-10-10):
  `buildLayeredImage`, a copy of only the tool's own files, and `fakeNss`
  for `nobody`.
- **The same Python as the dev shell.** `nix/import-python.nix` and the
  shared pin `nix/pkgs.nix` make the sandbox's OpenCASCADE the build the
  tests run against.
- **Only `tools/import/*.py` is copied:** the `tests/` fixtures, sidecars
  and bytecode stay out, and edits elsewhere don't rebuild the image.
- **Size and checks** (2026-10-10): about 1.1 GB, mostly OpenCASCADE.
  Checked: it runs as nobody, OCC imports, and the fixture and a 349-part
  real robot import through the broker's command (`deploy/smoke.sh`).

## Assertions
- [ ] No network tooling (curl, wget, ssh) and no package manager in
  `contents`.
- [ ] The container user is non-root.
- [ ] `/opt/import/step_import.py` exists, at the path `import_argv` runs.
- [ ] `deploy/smoke.sh` passes after any change here.
