#!/usr/bin/env python3
"""Writes tests/fixtures/simple_bot.step: the importer's known-answer robot,
laid out like a real 12 lb vertical spinner (user, 2026-10-10) so it also
drives like one.

Frame: x forward, y left, z up, floor at z = 0, mm. Every part is a box,
a cylinder or a straight prism, so each one's volume and centroid have a
closed form (tests/test_import.py checks them):

  base_plate   box   x -150..100  y -100..100  z  6..12
  rail_L/_R    box   x -150..130  y  90..100 / -100..-90  z 12..62  ("Rail")
  top_plate    box   x -150..60   y -100..100  z 62..65
  rear_bar     box   x -160..-150 y -100..100  z 12..52
  battery      box   x  -20..80   y  -20..20   z 12..42
  esc          box   x -130..-80  y  -25..25   z 12..27
  fork_L/_R    prism, side profile (120,2) (200,2) (120,40) in xz,
               y 30..55 / -55..-30                          ("Fork")
  wheel_L/_R   cylinder r 31.75 w 25, axis +y, centre (-30, +-120, 31.75)
                                                            ("Wheel")
  drive_motor_L/_R  cylinder r 14, axis +y, y 30..88 / -88..-30,
               centre line x -30 z 31.75                    ("DriveMotor")
  weapon       disc r 75 t 10, axis +y, centre (150, 0, 80) ("Weapon")

The vertical disc runs between the forks; the forks are the wedge and the
front skids. The axle is 30 mm behind centre and the CoM ~58 mm ahead of
it, so the robot rests on its wheels and fork tips. Re-run only to change
the fixture; the .step is checked in so tests never depend on this
script's output being stable.
"""

import os
import sys

from OCC.Core.BRepBuilderAPI import BRepBuilderAPI_MakeFace, BRepBuilderAPI_MakePolygon
from OCC.Core.BRepPrimAPI import (BRepPrimAPI_MakeBox, BRepPrimAPI_MakeCylinder,
                                  BRepPrimAPI_MakePrism)
from OCC.Core.gp import gp_Ax2, gp_Dir, gp_Pnt, gp_Trsf, gp_Vec
from OCC.Core.IFSelect import IFSelect_RetDone
from OCC.Core.STEPCAFControl import STEPCAFControl_Writer
from OCC.Core.STEPControl import STEPControl_AsIs
from OCC.Core.TDataStd import TDataStd_Name
from OCC.Core.TDocStd import TDocStd_Document
from OCC.Core.TopLoc import TopLoc_Location
from OCC.Core.XCAFDoc import XCAFDoc_DocumentTool


def name(label, text):
    TDataStd_Name.Set(label, text)


def located(x, y, z):
    t = gp_Trsf()
    t.SetTranslation(gp_Vec(x, y, z))
    return TopLoc_Location(t)


def box(x0, x1, y0, y1, z0, z1):
    return BRepPrimAPI_MakeBox(gp_Pnt(x0, y0, z0), x1 - x0, y1 - y0, z1 - z0).Shape()


def y_cylinder(r, y0, y1):
    """A cylinder on the y axis through the origin, from y0 to y1."""
    return BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0, y0, 0), gp_Dir(0, 1, 0)), r, y1 - y0).Shape()


def fork():
    """The fork's side profile at y = 0, extruded 25 mm along +y."""
    poly = BRepBuilderAPI_MakePolygon(gp_Pnt(120, 0, 2), gp_Pnt(200, 0, 2),
                                      gp_Pnt(120, 0, 40), True)
    face = BRepBuilderAPI_MakeFace(poly.Wire()).Face()
    return BRepPrimAPI_MakePrism(face, gp_Vec(0, 25, 0)).Shape()


# Products (one shape each, in its own frame), then where each instance of
# one is placed.
PRODUCTS = {
    "BasePlate": lambda: box(-150, 100, -100, 100, 6, 12),
    "Rail": lambda: box(-150, 130, 0, 10, 12, 62),
    "TopPlate": lambda: box(-150, 60, -100, 100, 62, 65),
    "RearBar": lambda: box(-160, -150, -100, 100, 12, 52),
    "Battery": lambda: box(-20, 80, -20, 20, 12, 42),
    "ESC": lambda: box(-130, -80, -25, 25, 12, 27),
    "Fork": fork,
    "Wheel": lambda: y_cylinder(31.75, -12.5, 12.5),
    "DriveMotor": lambda: y_cylinder(14, -29, 29),
    "Weapon": lambda: y_cylinder(75, -5, 5),
}
INSTANCES = (
    ("base_plate", "BasePlate", (0, 0, 0)),
    ("rail_L", "Rail", (0, 90, 0)),
    ("rail_R", "Rail", (0, -100, 0)),
    ("top_plate", "TopPlate", (0, 0, 0)),
    ("rear_bar", "RearBar", (0, 0, 0)),
    ("battery", "Battery", (0, 0, 0)),
    ("esc", "ESC", (0, 0, 0)),
    ("fork_L", "Fork", (0, 30, 0)),
    ("fork_R", "Fork", (0, -55, 0)),
    ("wheel_L", "Wheel", (-30, 120, 31.75)),
    ("wheel_R", "Wheel", (-30, -120, 31.75)),
    ("drive_motor_L", "DriveMotor", (-30, 59, 31.75)),
    ("drive_motor_R", "DriveMotor", (-30, -59, 31.75)),
    ("weapon", "Weapon", (150, 0, 80)),
)


def main(out):
    doc = TDocStd_Document("fixture")
    tool = XCAFDoc_DocumentTool.ShapeTool(doc.Main())
    asm = tool.NewShape()
    name(asm, "simple_bot")
    products = {}
    for label, make in PRODUCTS.items():
        products[label] = tool.AddShape(make(), False)
        name(products[label], label)
    for inst, ref, (x, y, z) in INSTANCES:
        comp = tool.AddComponent(asm, products[ref], located(x, y, z))
        name(comp, inst)
    tool.UpdateAssemblies()

    writer = STEPCAFControl_Writer()
    writer.SetNameMode(True)
    writer.Transfer(doc, STEPControl_AsIs)
    if writer.Write(out) != IFSelect_RetDone:
        sys.exit("write failed: " + out)


if __name__ == "__main__":
    here = os.path.dirname(os.path.abspath(__file__))
    main(os.path.join(here, "fixtures", "simple_bot.step"))
