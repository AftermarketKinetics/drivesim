"""What a tool is, what it gets to work with, and input validation.

Copied from the Combat Robot Analyzer (2026-10-09); `Ctx` is drivesim's.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ..config import Settings


class ToolError(Exception):
    """A failure the agent should read and act on (returned as is_error)."""


@dataclass
class Ctx:
    robot_id: str
    #: The robot's directory: robot.step and bundle/.
    robot_dir: Path
    settings: Settings
    store: Any  # db.Store; typed loosely to keep tools import-light
    #: Push a UI event to whoever is watching this robot.
    emit: Callable[[dict[str, Any]], Awaitable[None]]

    @property
    def bundle(self) -> Path:
        return self.robot_dir / "bundle"


@dataclass(frozen=True)
class Tool:
    name: str
    description: str
    input_schema: dict[str, Any]
    run: Callable[[Ctx, dict[str, Any]], Awaitable[Any]]

    def definition(self) -> dict[str, Any]:
        return {"name": self.name, "description": self.description, "input_schema": self.input_schema}


_TYPES = {"string": str, "number": (int, float), "integer": int, "boolean": bool,
          "array": list, "object": dict, "null": type(None)}


def validate(schema: dict[str, Any], value: Any, path: str = "input") -> None:
    """Enough JSON Schema for these tools: type, required, properties, items,
    enum, additionalProperties: false. Raises ToolError naming the field."""
    types = schema.get("type")
    if types is not None:
        names = types if isinstance(types, list) else [types]
        ok = any(isinstance(value, _TYPES[n]) and not (n in ("number", "integer") and isinstance(value, bool))
                 for n in names)
        if not ok:
            raise ToolError(f"{path}: expected {'/'.join(names)}")
    if "enum" in schema and value not in schema["enum"]:
        raise ToolError(f"{path}: must be one of {schema['enum']}")
    if isinstance(value, dict):
        props = schema.get("properties", {})
        for key in schema.get("required", []):
            if key not in value:
                raise ToolError(f"{path}.{key}: required")
        for key, sub in value.items():
            if key in props:
                validate(props[key], sub, f"{path}.{key}")
            elif schema.get("additionalProperties") is False:
                raise ToolError(f"{path}.{key}: unknown field")
            elif isinstance(schema.get("additionalProperties"), dict):
                validate(schema["additionalProperties"], sub, f"{path}.{key}")
    if isinstance(value, list) and "items" in schema:
        for i, item in enumerate(value):
            validate(schema["items"], item, f"{path}[{i}]")
