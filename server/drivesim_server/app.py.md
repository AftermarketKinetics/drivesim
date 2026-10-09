# app.py

## Function
The server's HTTP surface (docs/drivesim.md §3–4): robots, STEP upload, the
role conversation, live events, and the robot bundle the browser sim loads.
Adapted from the Combat Robot Analyzer's app (2026-10-09).

## Interface
All under authentik, and scoped to the caller's own robots: anyone else's
robot is a 404.

| Route | Does |
|---|---|
| `GET /api/me` | `{"username"}` |
| `GET /api/robots` | the caller's robots, newest first |
| `POST /api/robots` (multipart `file`) | 201 `{"id", "bytes"}`; starts the import; 413 over the cap |
| `GET /api/robots/{id}` | `id, name, status, error, checked, created` |
| `DELETE /api/robots/{id}` | 204; deletes the files and history |
| `GET /api/robots/{id}/messages` | the stored history |
| `POST /api/robots/{id}/messages` `{"text"}` | 202, runs the agent; 409 until imported |
| `GET /api/robots/{id}/events` | SSE: see `events.py.md` |
| `GET /api/robots/{id}/bundle/{rel}` | only `parts.json`, `robot.toml`, `meshes/NNN.stl` |
| `GET /api/motors.toml` | the motor library |
| `/` | `server/web` pages, mounted last |

Also `build_state(settings=None, model=None)`, so tests can inject their own
settings and a scripted model.

## Implementation
- **Upload:** a new robot gets its own directory, the STEP is streamed to
  `robot.step` up to the size cap, then `import_and_start` runs in the
  background.
- **`import_and_start`:**
  1. Creates `bundle/` with mode 0777, because the broker mounts it as the
     sandbox's only writable path.
  2. Runs the import.
  3. **On success:** sets the robot to `ready`, emits `status`, and notifies
     the agent ("N parts. Begin."), which starts the conversation.
  4. **On failure (a broker error, a non-zero exit or no `parts.json`):** sets
     `failed` with the error, and emits `status`.
- **Bundle files:** a fixed allowlist plus a regex for mesh names, so no
  path traversal reaches the filesystem. The tests send
  percent-encoded `../` variants.
- **The event stream:** `Cache-Control: no-cache, no-transform` and
  `X-Accel-Buffering: no`, so Cloudflare doesn't buffer it (the analyzer's
  finding).
- **Pages:** the static pages carry no login check of their own. On
  accelerate, Caddy's forward_auth covers every path.

## Assertions
- [ ] Every `/api/robots/{id}...` route goes through `owned`.
- [ ] Nothing outside a robot's `bundle/` allowlist is served.
- [ ] A failed import never crashes the server; it marks the robot `failed`.
