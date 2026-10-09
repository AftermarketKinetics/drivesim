# config.py

## Function
Every setting the deployment controls, read once from the environment.
Adapted from the Combat Robot Analyzer's pattern (2026-10-09).

## Interface
`Settings` (frozen) and `load_settings(env=None)`. Fields and `DRIVESIM_*`
variables, with development defaults (repo-relative):

| Field | Env | Default |
|---|---|---|
| `robots_root` | `ROBOTS_ROOT` | `server/var/robots` |
| `db_path` | `DB_PATH` | `server/var/drivesim.sqlite3` |
| `import_mode` | `IMPORT_MODE` | `local` (production: `broker`) |
| `broker_url` | `BROKER_URL` | `http://127.0.0.1:8711` |
| `import_python`, `import_script` | `IMPORT_PYTHON`, `IMPORT_SCRIPT` | `python3`, `tools/import/step_import.py` |
| `check_bin`, `motors_toml` | `CHECK_BIN`, `MOTORS_TOML` | `target/release/drivesim-check`, `data/motors.toml` |
| `model`, `effort` | `MODEL`, `EFFORT` | `claude-sonnet-5-5`, `medium` |
| `max_upload_bytes` | `MAX_UPLOAD_BYTES` | 95 MB |
| `import_timeout_s` | `IMPORT_TIMEOUT_S` | 600 |
| `web_dir` | `WEB_DIR` | `server/web` |
| `dev_user` | `DEV_USER` | unset |

## Implementation
The 95 MB upload cap is under Cloudflare's 100 MB free-plan body limit, which
every request to accelerate passes through (the analyzer's finding).
`claude-sonnet-5-5` is the user's choice (2026-10-09).

## Assertions
- [ ] Nothing else in the package reads `os.environ`.
- [ ] `dev_user` defaults to unset.
