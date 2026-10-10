"""The admin dashboard's API: every robot, for the deployment's admins
(user, 2026-10-10). Read only; the robot pages themselves let an admin
read and drive any robot (``viewable`` in app.py)."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from fastapi import APIRouter, Depends, Request

from .auth import admin_user
from .usage import day_bounds

router = APIRouter()


@router.get("/api/admin/robots")
def all_robots(request: Request, _admin: str = Depends(admin_user)) -> dict[str, Any]:
    """Every robot with its owner and lifetime usage, and each user's usage
    today against the daily budget."""
    s = request.app.state.s
    start, end = day_bounds(datetime.now(UTC), s["settings"].budget_tz)
    today = s["store"].used_by_user_since(start.timestamp())
    return {
        "robots": [{**r, "used": round(r["used"])} for r in s["store"].all_robots()],
        "users_today": {u: round(v) for u, v in sorted(today.items())},
        "budget": round(s["settings"].daily_budget),
        "resets_at": end.isoformat(),
    }
