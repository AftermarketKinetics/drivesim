"""Who may touch a robot: FastAPI dependencies that resolve a robot id
from the path, or 404 (never 403, so ids don't leak).

- ``owned``: the owner only. For changing a robot.
- ``viewable``: the owner or an admin (user, 2026-10-10). For reading it:
  its page, conversation and events.
- ``drivable``: viewable, or public (user, 2026-10-10: public robots are
  AI opponents for everyone). For its bundle only; a public robot's
  conversation stays private.
"""

from __future__ import annotations

from fastapi import Depends, HTTPException, Request

from .auth import current_user, is_admin


def _robot(robot_id: str, request: Request) -> dict | None:
    if len(robot_id) != 32:
        return None
    return request.app.state.s["store"].robot(robot_id)


def owned(robot_id: str, request: Request, user: str = Depends(current_user)) -> str:
    r = _robot(robot_id, request)
    if not r or r["username"] != user:
        raise HTTPException(status_code=404, detail="no such robot")
    return robot_id


def viewable(robot_id: str, request: Request, user: str = Depends(current_user)) -> str:
    r = _robot(robot_id, request)
    if not r or not (r["username"] == user or is_admin(request, user)):
        raise HTTPException(status_code=404, detail="no such robot")
    return robot_id


def drivable(robot_id: str, request: Request, user: str = Depends(current_user)) -> str:
    r = _robot(robot_id, request)
    if not r or not (r["username"] == user or r["public"] or is_admin(request, user)):
        raise HTTPException(status_code=404, detail="no such robot")
    return robot_id
