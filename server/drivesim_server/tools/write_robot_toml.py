"""write_robot_toml: replace the robot file, then check it with the sim's
own loader and physics."""

from __future__ import annotations

from typing import Any

from ..check import check_bundle
from .base import Ctx, Tool, ToolError

MAX_CHARS = 20_000


async def run(ctx: Ctx, args: dict[str, Any]) -> dict[str, Any]:
    text = args["robot_toml"]
    if len(text) > MAX_CHARS:
        raise ToolError(f"robot.toml is over {MAX_CHARS} characters")
    if not (ctx.bundle / "parts.json").is_file():
        raise ToolError("the STEP has not been imported yet")
    (ctx.bundle / "robot.toml").write_text(text)
    report = await check_bundle(ctx.settings, ctx.bundle)
    ctx.store.set_checked(ctx.robot_id, bool(report.get("ok")), report.get("name"))
    await ctx.emit({"type": "robot", "report": report})
    return report


TOOL = Tool(
    name="write_robot_toml",
    description=(
        "Write the whole robot.toml (replacing any previous one) and check it: the sim's own "
        "loader and physics load it, drop the robot on the floor and report total mass, CoM, "
        "inertia, each wheel and weapon (axis, radius, no-load speed) and how it rests (tilt, "
        "which bodies touch the floor). On a mistake it returns ok=false and the error. The "
        "robot is drivable once this returns ok=true."),
    input_schema={"type": "object", "properties": {"robot_toml": {"type": "string"}},
                  "required": ["robot_toml"], "additionalProperties": False},
    run=run,
)
