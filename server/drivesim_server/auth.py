"""Who is calling: the authentik username Caddy forwards.

Copied from the Combat Robot Analyzer (2026-10-09). Caddy strips every
client-supplied ``X-authentik-*`` header and re-adds them only from the
outpost's verified response, so a header that reaches us is trustworthy as
long as the server is only reachable through Caddy (it publishes no port).
Admins (user, 2026-10-10) are a list of those usernames in the settings.
"""

from __future__ import annotations

import re

from fastapi import Depends, Header, HTTPException, Request

_USERNAME = re.compile(r"^[A-Za-z0-9._@+-]{1,150}$")


def current_user(request: Request, x_authentik_username: str | None = Header(default=None)) -> str:
    """FastAPI dependency: the signed-in username, or 401.

    In development (``DRIVESIM_DEV_USER`` set) a request with no header is
    that user, so the server runs on localhost without authentik.
    """
    name = x_authentik_username or request.app.state.s["settings"].dev_user
    if not name or not _USERNAME.match(name):
        raise HTTPException(status_code=401, detail="not signed in")
    return name


def is_admin(request: Request, user: str) -> bool:
    """Whether `user` is one of the deployment's admins (``DRIVESIM_ADMINS``)."""
    return user in request.app.state.s["settings"].admins


def admin_user(request: Request, user: str = Depends(current_user)) -> str:
    """FastAPI dependency: the signed-in username if an admin, else 403."""
    if not is_admin(request, user):
        raise HTTPException(status_code=403, detail="admins only")
    return user
