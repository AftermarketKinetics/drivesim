"""Public robots as AI opponents (user, 2026-10-10): an owner tags a
robot public with its weight class, and every signed-in user's sim page
lists the public robots for the AI to drive."""

from __future__ import annotations

from typing import Any

from fastapi import APIRouter, Depends, HTTPException, Request
from pydantic import BaseModel

from .access import owned
from .auth import current_user
from .db import WEIGHT_CLASSES

router = APIRouter()


class Publish(BaseModel):
    public: bool
    #: One of WEIGHT_CLASSES; required when public, 0 or a class otherwise.
    weight_lb: int = 0


@router.put("/api/robots/{robot_id}/public")
def publish(body: Publish, request: Request, robot_id: str = Depends(owned)) -> dict[str, Any]:
    """Make the caller's robot public (an AI opponent for everyone) or
    private again. Only a drivable robot may be made public."""
    store = request.app.state.s["store"]
    if body.weight_lb not in WEIGHT_CLASSES and (body.public or body.weight_lb != 0):
        raise HTTPException(status_code=422,
                            detail=f"weight class must be one of {', '.join(map(str, WEIGHT_CLASSES))} lb")
    r = store.robot(robot_id)
    if body.public and not (r["status"] == "ready" and r["checked"]):
        raise HTTPException(status_code=409, detail="only a robot that is ready to drive can be public")
    store.set_public(robot_id, body.public, body.weight_lb)
    return {"public": body.public, "weight_lb": body.weight_lb}


@router.get("/api/opponents")
def opponents(request: Request, user: str = Depends(current_user)) -> list[dict[str, Any]]:
    """Every public, drivable robot, by weight class then name, with the
    caller's own marked. Owners' usernames are not listed."""
    return [{"id": r["id"], "name": r["name"], "weight_lb": r["weight_lb"],
             "mine": r["username"] == user}
            for r in request.app.state.s["store"].public_robots()]
