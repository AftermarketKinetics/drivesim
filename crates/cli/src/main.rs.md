# main.rs (drivesim-check)

## Function

The command the server runs after every `robot.toml` write: it loads a
robot bundle exactly as the sim does and reports it. The role
conversation then checks its work against the same code the sim runs.

## Interface

```
drivesim-check BUNDLE_DIR MOTORS_TOML
```

- `BUNDLE_DIR` holds `parts.json`, `robot.toml` and `meshes/`.
- Prints one JSON object on stdout (see `report.rs.md`).
- Exit codes: 0 when the robot loads, 1 when it doesn't (with
  `{"ok": false, "error"}` printed), 2 on a usage error.

## Implementation

All file access is here. `report` takes bytes and a mesh-fetch closure,
like the physics crate.

## Assertions

- A missing file is reported as JSON with exit 1, never a panic.
