# app.py

## Function
The server's HTTP surface (docs/drivesim.md §3–4): robots, STEP upload, the
role conversation, live events, and the robot bundle the browser sim loads.
Adapted from the Combat Robot Analyzer's app (2026-10-09).

## Interface
All under authentik, and scoped to the caller's own robots: anyone else's
robot is a 404. **Admins** (`Settings.admins`, user 2026-10-10) may also
read and drive every robot: the read routes use `viewable` (owner or
admin), while `DELETE` and `POST .../messages` stay `owned`, so an admin
can neither delete a robot nor spend its owner's budget.

| Route | Does |
|---|---|
| `GET /api/me` | `{"username", "admin"}` |
| `GET /api/usage` | today's `{"used", "budget", "resets_at", "tz"}`, in input-token equivalents |
| `GET /api/robots` | the caller's robots, newest first |
| `POST /api/robots` (multipart `file`) | 201 `{"id", "bytes"}`; starts the import; 413 over the cap; 429 when today's budget is spent |
| `GET /api/robots/{id}` | `id, name, status, error, checked, created, owner, mine` (viewable) |
| `DELETE /api/robots/{id}` | 204; deletes the files and history |
| `GET /api/robots/{id}/messages` | the stored history (viewable) |
| `POST /api/robots/{id}/messages` `{"text"}` | 202, runs the agent; 409 until imported; 429 when the owner's budget is spent |
| `GET /api/robots/{id}/events` | SSE: see `events.py.md` (viewable) |
| `GET /api/robots/{id}/bundle/{rel}` | only `parts.json`, `robot.toml`, `meshes/NNN.stl` (viewable) |
| `GET /api/admin/robots` | admins only, 403 otherwise: see `admin.py.md` (router included here) |
| `GET /api/motors.toml` | the motor library |
| `/` | `server/web` pages, mounted last |

Also `build_state(settings=None, model=None)`, so tests can inject their own
settings and a scripted model, and `budget(s, user)` and
`require_budget(s, user)`.

**Static files** (`WebFiles`) are served `no-cache` with an ETag from each
file's content, hashed once and kept. Nix images give every file a 1970
mtime, so Starlette's default left pages cached for years and same-size
rebuilds came back 304 (the analyzer's lesson, 2026-10-08).

**The daily budget** (user, 2026-10-10): uploads and messages are refused
with 429 once the user's weighted usage since midnight Pacific reaches
`daily_budget`. The detail names the usage, the reset and that set-up
robots can still be driven. It is checked before work starts, so a reply
already running finishes and can overshoot. Driving and bundle fetches
cost nothing and are never limited.

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
- [ ] Every `/api/robots/{id}...` route goes through `owned` or `viewable`;
  anything that changes a robot or spends budget goes through `owned`.
  (Changed deliberately 2026-10-10 for admin access; it was `owned` only.)
- [ ] Nothing outside a robot's `bundle/` allowlist is served.
- [ ] A failed import never crashes the server; it marks the robot `failed`.
