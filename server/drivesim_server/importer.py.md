# importer.py

## Function
Runs the STEP importer on one robot (docs/drivesim.md §4):
- **production:** in a gVisor sandbox through drivesim's broker, because a
  STEP upload is untrusted input to OpenCASCADE (user, 2026-10-09);
- **development:** as a local subprocess.

## Interface
- `STEP_NAME` is `"robot.step"` and `BUNDLE` is `"bundle"`, both relative to
  the robot's directory.
- `ImportResult(code, stdout, stderr)`, with `.ok` true when the code is 0.
- `async run_import(settings, robot_id, robot_dir) -> ImportResult`:
  - with `import_mode == "broker"`, it POSTs `{broker}/imports/<id>/run`;
  - with `"local"`, it runs `import_python import_script robot.step bundle`;
  - any other mode raises `ValueError`.

## Implementation
- **Broker mode:** the HTTP timeout is the import timeout plus 60 s. A
  non-200 reply raises, with the broker's error text.
- **Local mode:**
  - On a timeout the process is killed and the result is code 137.
  - Output is kept to the last 1 MiB of stdout and 8 KiB of stderr.
  - `import_python` must be a Python that has pythonocc: the nix shell's,
    not the uv venv's. `make server-dev` passes it explicitly.

## Assertions
- [ ] The server process never imports pythonocc or parses STEP itself.
- [ ] Local mode is never used in production. The deployed image sets
  `DRIVESIM_IMPORT_MODE=broker`.
