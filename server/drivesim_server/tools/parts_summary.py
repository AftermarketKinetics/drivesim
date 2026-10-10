"""parts_summary: the imported parts, compact enough to read in one go.

Real assemblies are mostly hardware: the first real robot (2026-10-10, 349
parts) had 100 identical stator plates and dozens of identical screws, and
one line per part overflowed the loop's result cap, so the agent never saw
its weapon. Parts of a product placed more than `GROUP_OVER` times share one
line, and lines come biggest first.
"""

from __future__ import annotations

import json
from typing import Any

from .base import Ctx, Tool, ToolError

#: Instances of one product beyond this count share a line. Two stay apart:
#: a left and right wheel are told apart by where they are.
GROUP_OVER = 2


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


def _group_line(ps: list[dict[str, Any]]) -> str:
    """One line for many instances of one product."""
    ids = ",".join(str(p["id"]) for p in ps)
    lo = [min(p["bbox_m"][0][k] for p in ps) * 1000 for k in range(3)]
    hi = [max(p["bbox_m"][1][k] for p in ps) * 1000 for k in range(3)]
    span = " to ".join("(" + ",".join(f"{c:.0f}" for c in v) + ")" for v in (lo, hi))
    return (f"{len(ps)} x product {ps[0]['product']!r}, {ps[0]['volume_m3'] * 1e6:.1f} cm3 each, "
            f"spread over {span} mm; ids {ids}")


def _bbox(parts: list[dict[str, Any]]) -> str:
    lo = [min(p["bbox_m"][0][k] for p in parts) * 1000 for k in range(3)]
    hi = [max(p["bbox_m"][1][k] for p in parts) * 1000 for k in range(3)]
    return " to ".join("(" + ",".join(f"{c:.0f}" for c in v) + ")" for v in (lo, hi)) + " mm"


async def run(ctx: Ctx, args: dict[str, Any]) -> dict[str, Any]:
    path = ctx.bundle / "parts.json"
    if not path.is_file():
        raise ToolError("the STEP has not been imported yet")
    doc = json.loads(path.read_text())
    every = [p for p in doc["parts"] if p["bbox_m"][0]]
    parts = doc["parts"]
    if args.get("contains"):
        needle = args["contains"].lower()
        parts = [p for p in parts
                 if needle in f"{p['path']} {p['instance']} {p['product']}".lower()]
    by_product: dict[str, list[dict[str, Any]]] = {}
    for p in parts:
        by_product.setdefault(p["product"], []).append(p)
    rows: list[tuple[float, str]] = []
    for ps in by_product.values():
        if len(ps) > GROUP_OVER:
            rows.append((sum(p["volume_m3"] for p in ps), _group_line(ps)))
        else:
            rows.extend((p["volume_m3"], _line(p)) for p in ps)
    rows.sort(key=lambda r: -r[0])
    return {"robot_id": ctx.robot_id, "source": doc["source"], "frame": doc["frame"], "count": len(parts),
            "robot_bbox": _bbox(every) if every else None,
            "note": ("biggest first; more than two instances of one product share a line. "
                     "The STEP origin is arbitrary: it is not the floor."),
            "parts": [line for _, line in rows]}


TOOL = Tool(
    name="parts_summary",
    description=(
        "List the imported parts of the robot's STEP, one line each: id, instance and product "
        "names, volume, bounding box and centre (mm, in the STEP's own axes), and for parts with "
        "revolved faces their rotation axis, outer radius, width, how much of the surface is "
        "curved about / flat across that axis, and how far the part's centre is from it (a "
        "balanced rotor: ~0). Biggest parts first; more than two instances of one product "
        "(screws, bearings, magnets) share one line with their ids. Also gives the whole "
        "robot's bounding box and its robot_id, for the user to quote when asking for help. Optional `contains` filters by a substring of the instance, "
        "product or tree path."),
    input_schema={"type": "object", "properties": {"contains": {"type": "string"}},
                  "additionalProperties": False},
    run=run,
)
