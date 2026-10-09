"""The role conversation's tools: one module per tool, collected into ``TOOLS``.

The agent reads the imported parts, the motor library and the current robot
file, and writes the robot file (which is checked on every write). It never
touches the STEP, runs the importer or runs a shell. Inputs are validated
before a tool runs, because streamed tool input is not validated by the API.
"""

from __future__ import annotations

from . import list_motors, parts_summary, show_robot_toml, write_robot_toml
from .base import Ctx, Tool, ToolError, validate

TOOLS: dict[str, Tool] = {
    t.name: t
    for t in (parts_summary.TOOL, list_motors.TOOL, show_robot_toml.TOOL, write_robot_toml.TOOL)
}

__all__ = ["TOOLS", "Ctx", "Tool", "ToolError", "validate"]
