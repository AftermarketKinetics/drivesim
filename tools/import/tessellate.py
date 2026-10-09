"""Turns a solid into triangles in metres and writes them as binary STL."""

import struct

from OCC.Core.BRep import BRep_Tool
from OCC.Core.BRepMesh import BRepMesh_IncrementalMesh
from OCC.Core.TopAbs import TopAbs_FACE, TopAbs_REVERSED
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.TopLoc import TopLoc_Location
from OCC.Core.TopoDS import topods

MM = 1e-3
# Chord deviation, mm, and angle, rad: fine enough that a 43 mm wheel reads
# within 0.1 mm of its radius, coarse enough for a whole robot in seconds.
LINEAR_MM = 0.1
ANGULAR_RAD = 0.3


def triangles(solid, linear_mm=LINEAR_MM, angular_rad=ANGULAR_RAD):
    """Outward-wound triangles ((x,y,z) x3) of the solid, in metres.

    Faces OCC marks REVERSED have their winding swapped, so every normal
    points out of the solid; collision shapes and back-face culling both
    depend on that.
    """
    BRepMesh_IncrementalMesh(solid, linear_mm, False, angular_rad, True)
    out = []
    exp = TopExp_Explorer(solid, TopAbs_FACE)
    while exp.More():
        face = topods.Face(exp.Current())
        loc = TopLoc_Location()
        tri = BRep_Tool.Triangulation(face, loc)
        if tri is not None:
            trsf = loc.Transformation()
            pts = []
            for i in range(1, tri.NbNodes() + 1):
                p = tri.Node(i)
                if not loc.IsIdentity():
                    p = p.Transformed(trsf)
                pts.append((p.X() * MM, p.Y() * MM, p.Z() * MM))
            flip = face.Orientation() == TopAbs_REVERSED
            for i in range(1, tri.NbTriangles() + 1):
                a, b, c = tri.Triangle(i).Get()
                if flip:
                    b, c = c, b
                out.append((pts[a - 1], pts[b - 1], pts[c - 1]))
        exp.Next()
    return out


def write_stl(path, tris):
    """Binary STL; normals from the winding (right-hand rule)."""
    with open(path, "wb") as f:
        f.write(b"drivesim import, metres".ljust(80, b" "))
        f.write(struct.pack("<I", len(tris)))
        for a, b, c in tris:
            u = [b[i] - a[i] for i in range(3)]
            v = [c[i] - a[i] for i in range(3)]
            n = (u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2],
                 u[0] * v[1] - u[1] * v[0])
            length = sum(x * x for x in n) ** 0.5 or 1.0
            f.write(struct.pack("<3f", *(x / length for x in n)))
            for p in (a, b, c):
                f.write(struct.pack("<3f", *p))
            f.write(b"\0\0")
