"""Finds the axis a solid is turned about, if it has one: the evidence the
roles skill uses to spot wheels, weapons and hubs."""

import math

from OCC.Core.BRepAdaptor import BRepAdaptor_Surface
from OCC.Core.BRepGProp import brepgprop
from OCC.Core.GeomAbs import (GeomAbs_Cone, GeomAbs_Cylinder, GeomAbs_Plane,
                              GeomAbs_SurfaceOfRevolution, GeomAbs_Torus)
from OCC.Core.GProp import GProp_GProps
from OCC.Core.TopAbs import TopAbs_FACE
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.TopoDS import topods

MM = 1e-3
# Two axes are the same line within these tolerances (mm, |sin angle|).
POS_TOL_MM = 0.01
DIR_TOL = 1e-6


def _area(face):
    g = GProp_GProps()
    brepgprop.SurfaceProperties(face, g)
    return g.Mass()


def _axis_of(surf):
    kind = surf.GetType()
    if kind == GeomAbs_Cylinder:
        return surf.Cylinder().Axis()
    if kind == GeomAbs_Cone:
        return surf.Cone().Axis()
    if kind == GeomAbs_Torus:
        return surf.Torus().Axis()
    if kind == GeomAbs_SurfaceOfRevolution:
        return surf.AxeOfRevolution()
    return None


def _same_line(a, b):
    """a, b: (point, unit dir) tuples in mm."""
    (pa, da), (pb, db) = a, b
    cross = (da[1] * db[2] - da[2] * db[1], da[2] * db[0] - da[0] * db[2],
             da[0] * db[1] - da[1] * db[0])
    if math.sqrt(sum(c * c for c in cross)) > DIR_TOL:
        return False
    d = [pb[i] - pa[i] for i in range(3)]
    t = sum(d[i] * da[i] for i in range(3))
    off = [d[i] - t * da[i] for i in range(3)]
    return math.sqrt(sum(o * o for o in off)) < POS_TOL_MM


def rotation_axis(solid):
    """The solid's dominant axis of revolution, or None if it has no
    revolved faces.

    Revolved faces (cylinders, cones, tori, surfaces of revolution) are
    grouped by the line they turn about; the group with the most area
    wins. Two shares of the whole surface are reported separately, because
    together they mislead: a flat plate with one bolt hole has huge faces
    square to the hole's axis (Momentum's floor scored 89% on a combined
    score, 2026-10-09).

    - `curved_fraction`: the winning group's revolved area. High for a
      tyre or a drum, low for a disc (mostly flat faces) or a plate.
    - `cap_fraction`: planar faces square to the axis. High for a disc or
      a plate; so it only means "round" together with `com_offset_m`
      (see `distance_to_axis`), which a balanced rotor keeps near zero.

    The caller decides what counts; this only reports.

    Returns {"point_m", "dir", "curved_fraction", "cap_fraction"}, with
    `dir` a unit vector whose largest component is positive (a canonical
    sign).
    """
    groups = []          # [line, curved_area]
    planes = []          # (normal, area)
    total = 0.0
    exp = TopExp_Explorer(solid, TopAbs_FACE)
    while exp.More():
        face = topods.Face(exp.Current())
        area = _area(face)
        total += area
        surf = BRepAdaptor_Surface(face, True)
        ax = _axis_of(surf)
        if ax is not None:
            p, d = ax.Location(), ax.Direction()
            line = ((p.X(), p.Y(), p.Z()), (d.X(), d.Y(), d.Z()))
            for g in groups:
                if _same_line(g[0], line):
                    g[1] += area
                    break
            else:
                groups.append([line, area])
        elif surf.GetType() == GeomAbs_Plane:
            n = surf.Plane().Axis().Direction()
            planes.append(((n.X(), n.Y(), n.Z()), area))
        exp.Next()
    if not groups or total <= 0.0:
        return None
    (point, d), curved = max(groups, key=lambda g: g[1])
    caps = sum(a for n, a in planes
               if abs(abs(sum(n[i] * d[i] for i in range(3))) - 1.0) < 1e-6)
    big = max(range(3), key=lambda i: abs(d[i]))
    sign = 1.0 if d[big] > 0 else -1.0
    return {
        "point_m": [x * MM for x in point],
        "dir": [sign * x for x in d],
        "curved_fraction": curved / total,
        "cap_fraction": caps / total,
    }


def distance_to_axis(axis, point_m):
    """Distance (m) from a point to the axis line: for a part's centre of
    volume, ~0 when the part is balanced about that axis."""
    p, d = axis["point_m"], axis["dir"]
    v = [point_m[i] - p[i] for i in range(3)]
    t = sum(v[i] * d[i] for i in range(3))
    return math.sqrt(max(0.0, sum(x * x for x in v) - t * t))


def radial_extent(axis, points_m):
    """Outer radius and the span along the axis of a cloud of points (m),
    for an axis from `rotation_axis`. Taken from the mesh rather than the
    B-rep so teeth, lugs and other non-round features count."""
    p, d = axis["point_m"], axis["dir"]
    r_max, lo, hi = 0.0, math.inf, -math.inf
    for q in points_m:
        v = [q[i] - p[i] for i in range(3)]
        t = sum(v[i] * d[i] for i in range(3))
        r = math.sqrt(max(0.0, sum(x * x for x in v) - t * t))
        r_max = max(r_max, r)
        lo, hi = min(lo, t), max(hi, t)
    return {"radius_m": r_max, "width_m": hi - lo,
            "centre_m": [p[i] + d[i] * (lo + hi) / 2 for i in range(3)]}
