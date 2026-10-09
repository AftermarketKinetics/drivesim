"""list_motors: the motor library robot.toml can name."""

from __future__ import annotations

import tomllib
from typing import Any

from .base import Ctx, Tool


async def run(ctx: Ctx, args: dict[str, Any]) -> dict[str, Any]:
    lib = tomllib.loads(ctx.settings.motors_toml.read_text())["motor"]
    return {"motors": [{"id": k, "kv_rpm_per_v": m["kv_rpm_per_v"], "i_max_a": m["i_max_a"],
                        "provisional": m.get("provisional", []), "source": m.get("source", "")}
                       for k, m in sorted(lib.items())]}


TOOL = Tool(
    name="list_motors",
    description=("The motor library: ids robot.toml's `motor` field can name, with kv, current "
                 "limit, which numbers are placeholders, and their source. Any number can be "
                 "overridden per rotor with a [wheel.motor_override] / [weapon.motor_override] table."),
    input_schema={"type": "object", "properties": {}, "additionalProperties": False},
    run=run,
)
