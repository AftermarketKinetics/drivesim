#!/usr/bin/env python3
"""STEP assembly -> parts directory (docs/drivesim.md §4).

    python tools/import/step_import.py robot.step out/robot-parts/

Writes `parts.json` and `meshes/NNN.stl`. Geometry only: no roles, no
materials. Everything is in SI units, in the STEP's own assembly frame.
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from occ_load import load                                   # noqa: E402
from mass_props import mass_props                           # noqa: E402
from rotation_axis import (distance_to_axis, radial_extent,  # noqa: E402
                           rotation_axis)
from tessellate import triangles, write_stl                 # noqa: E402

FORMAT = 1


def import_step(step_path, out_dir):
    os.makedirs(os.path.join(out_dir, "meshes"), exist_ok=True)
    parts = []
    for i, leaf in enumerate(load(step_path)):
        tris = triangles(leaf.shape)
        mesh = "meshes/%03d.stl" % i
        write_stl(os.path.join(out_dir, mesh), tris)
        pts = [p for t in tris for p in t]
        lo = [min(p[k] for p in pts) for k in range(3)] if pts else None
        hi = [max(p[k] for p in pts) for k in range(3)] if pts else None
        props = mass_props(leaf.shape)
        axis = rotation_axis(leaf.shape)
        if axis is not None and pts:
            axis.update(radial_extent(axis, pts))
            axis["com_offset_m"] = distance_to_axis(axis, props["com_m"])
        parts.append({
            "id": i,
            "instance": leaf.instance,
            "product": leaf.product,
            "path": leaf.path,
            "color": list(leaf.color) if leaf.color else None,
            "mesh": mesh,
            "triangles": len(tris),
            "bbox_m": [lo, hi],
            **props,
            "rotation_axis": axis,
        })
    doc = {
        "format": FORMAT,
        "source": os.path.basename(step_path),
        "units": "SI: m, m^3; inertia_m5 is kg m^2 at 1 kg/m^3",
        "frame": "the STEP assembly frame, unchanged",
        "parts": parts,
    }
    with open(os.path.join(out_dir, "parts.json"), "w") as f:
        json.dump(doc, f, indent=1)
    return doc


def main(argv):
    if len(argv) != 3:
        sys.exit(__doc__)
    doc = import_step(argv[1], argv[2])
    for p in doc["parts"]:
        ax = p["rotation_axis"]
        print("%3d %-24s %-16s %8.1f cm3  %s" % (
            p["id"], p["instance"], p["product"], p["volume_m3"] * 1e6,
            "axis %s r %.1f mm, curved %.0f%% caps %.0f%%, CoM off %.1f mm"
            % ([round(x, 3) for x in ax["dir"]], ax["radius_m"] * 1e3,
               ax["curved_fraction"] * 100, ax["cap_fraction"] * 100,
               ax["com_offset_m"] * 1e3) if ax else ""))


if __name__ == "__main__":
    main(sys.argv)
