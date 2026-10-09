"""parts_summary: the imported parts, compact enough to read in one go."""

from __future__ import annotations

import json
from typing import Any

from .base import Ctx, Tool, ToolError


def _line(p: dict[str, Any]) -> str:
    lo, hi = p["bbox_m"]
    size = "x".join(f"{(hi[k] - lo[k]) * 1000:.0f}" for k in range(3)) if lo and hi else "?"
    com = ",".join(f"{c * 1000:.0f}" for c in p["com_m"])
    line = (f"{p['id']}: {p['instance']!r} (product {p['product']!r}) "
            f"{p['volume_m3'] * 1e6:.1f} cm3, bbox {size} mm, com ({com}) mm")
    ax = p.get("rotation_axis")
    if ax:
        d = ",".join(f"{x:.2f}" for x in ax["dir"])
        line += (f"; axis ({d}) r {ax['radius_m'] * 1000:.1f} mm w {ax['width_m'] * 1000:.1f} mm, "
                 f"curved {ax['curved_fraction']:.0%} caps {ax['cap_fraction']:.0%}, "
                 f"com off axis {ax['com_offset_m'] * 1000:.1f} mm")
    return line


async def run(ctx: Ctx, args: dict[str, Any]) -> dict[str, Any]:
    path = ctx.bundle / "parts.json"
    if not path.is_file():
        raise ToolError("the STEP has not been imported yet")
    doc = json.loads(path.read_text())
    parts = doc["parts"]
    if args.get("contains"):
        needle = args["contains"].lower()
        parts = [p for p in parts if needle in p["path"].lower()]
    return {"source": doc["source"], "frame": doc["frame"], "count": len(parts),
            "parts": [_line(p) for p in parts]}


TOOL = Tool(
    name="parts_summary",
    description=(
        "List the imported parts of the robot's STEP, one line each: id, instance and product "
        "names, volume, bounding box and centre (mm, in the STEP's own axes), and for parts with "
        "revolved faces their rotation axis, outer radius, width, how much of the surface is "
        "curved about / flat across that axis, and how far the part's centre is from it (a "
        "balanced rotor: ~0). Optional `contains` filters by a substring of the tree path."),
    input_schema={"type": "object", "properties": {"contains": {"type": "string"}},
                  "additionalProperties": False},
    run=run,
)
