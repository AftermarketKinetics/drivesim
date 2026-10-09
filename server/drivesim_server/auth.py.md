# auth.py

## Function
Answers "who is calling": the authentik username that Caddy's forward_auth
puts on each request. It is the only identity the server has; every robot is
owned by one such username (owner-only, user 2026-10-09). Copied from the
Combat Robot Analyzer (2026-10-09), plus a development fallback.

## Interface
- `current_user(request, x_authentik_username: str | None = Header(None)) -> str`: FastAPI dependency; returns the username or raises `HTTPException(401, "not signed in")`.
- Accepted usernames match `^[A-Za-z0-9._@+-]{1,150}$`.
- Used by `app.py` on every `/api` route, directly or through `owned`.

## Implementation
- Caddy strips every client-supplied `X-authentik-*` header and re-adds them only from the outpost's verified response, so a header that arrives is trustworthy *as long as the server is only reachable through Caddy*: it publishes no port.
- **Development fallback** (drivesim's change to the analyzer's version): with `Settings.dev_user` set (env `DRIVESIM_DEV_USER`, which `make server-dev` sets for localhost), a request with no header is that user. A header still wins.

## Assertions
- [ ] A missing or malformed header yields 401 unless `DRIVESIM_DEV_USER` is set; the deployed image never sets it.
- [ ] No other header or cookie is consulted for identity.
- [ ] The server is never exposed except behind Caddy's forward_auth (otherwise the header can be forged).
