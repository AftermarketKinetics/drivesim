"""show_robot_toml: the robot file as it stands."""

from __future__ import annotations

from typing import Any

from .base import Ctx, Tool


async def run(ctx: Ctx, args: dict[str, Any]) -> dict[str, Any]:
    path = ctx.bundle / "robot.toml"
    return {"robot_toml": path.read_text() if path.is_file() else None}


TOOL = Tool(
    name="show_robot_toml",
    description="The current robot.toml text, or null if none has been written yet.",
    input_schema={"type": "object", "properties": {}, "additionalProperties": False},
    run=run,
)
