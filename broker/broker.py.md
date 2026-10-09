# broker.py

## Function
drivesim's import broker, and the only process that holds the Docker socket
(docs/drivesim.md §4). The server asks it to import one robot's STEP in a
fresh gVisor sandbox (user, 2026-10-09: its own broker). The broker builds
exactly one command, so a compromised server can at most run the importer on
some robot's files.

## Interface
- **HTTP, POST only:**
  - `POST /imports/<32 hex>/run` with `{"timeout": s}` returns 200
    `{"code", "stdout" (last 1 MiB), "stderr" (last 8 KiB)}`. The timeout
    defaults to 600 and is clamped to 10..1800.
  - Anything else gets `404 {"error": "not found"}`.
  - An exception gets `500 {"error"}`.
- **Pure helpers:** `config(env)`, `clamp_timeout(t)`,
  `import_argv(robot_id, timeout, cfg)`, `run_import(...)`, and
  `make_handler(cfg, gate, runner)`.
- **Environment:**

  | Variable | Default |
  |---|---|
  | `DRIVESIM_ROBOTS_HOST_ROOT` | required |
  | `DRIVESIM_IMPORT_IMAGE` | `drivesim-import:latest` |
  | `DRIVESIM_SANDBOX_RUNTIME` | `runsc` |
  | `DRIVESIM_SANDBOX_CPUS` | `2` |
  | `DRIVESIM_SANDBOX_MEMORY` | `4g` |
  | `DRIVESIM_BROKER_BIND` | `127.0.0.1:8711` |
  | `DRIVESIM_BROKER_PARALLEL` | `2` (concurrent imports) |

- **Run:** `python3 broker.py`. Standard library only.

## Implementation
- **Adapted from** the analyzer's cra-broker (2026-10-09). That broker
  `exec`s allow-listed commands into long-lived sessions; this one runs a
  single `docker run --rm` per import, so it needs no allowlist and no
  reaper.
- **Sandbox flags:**
  - `--runtime runsc`, `--network=none`, `--read-only`;
  - `--tmpfs /tmp`, `--cap-drop=ALL`, `no-new-privileges`;
  - `--pids-limit=256`, plus the CPU and memory caps.
- **Mounts:** `robot.step` read-only and `bundle/` writable. The sources
  are paths on the *host*, because the Docker daemon resolves bind mounts.
- **Inside the sandbox:** `umask 0` (under gVisor the default 022 left
  files the server couldn't write; the analyzer found this), then
  `timeout -s KILL`. Killing the docker CLI on the broker side would leave
  the import running.
- **Concurrency:** a semaphore limits imports to `PARALLEL` at once.
- **Logging:** requests are logged to stderr.

## Assertions
- [ ] The only command built is `import_argv`'s, with the robot id checked
  as 32 hex characters before any docker call.
- [ ] Every container runs with `--network=none`, `--read-only`,
  `--cap-drop=ALL` and the gVisor runtime.
- [ ] The only host paths mounted are that robot's `robot.step` (read-only)
  and `bundle/`.
- [ ] No environment (in particular no API key) is passed into a container.
