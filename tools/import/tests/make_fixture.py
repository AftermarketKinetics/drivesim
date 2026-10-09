#!/usr/bin/env python3
"""Writes tests/fixtures/simple_bot.step: the importer's known-answer robot.

A 200 x 160 x 40 mm chassis box, one "Wheel" product (r 30, w 20, axis +y)
placed twice as wheel_L / wheel_R touching the floor, and a "Weapon" disc
(r 80, t 6, axis +z) above the chassis. Frame: x forward, y left, z up,
floor at z = 0, mm. Re-run only to change the fixture; the .step is
checked in so tests never depend on this script's output being stable.
"""

import os
import sys

from OCC.Core.BRepPrimAPI import BRepPrimAPI_MakeBox, BRepPrimAPI_MakeCylinder
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


def main(out):
    doc = TDocStd_Document("fixture")
    tool = XCAFDoc_DocumentTool.ShapeTool(doc.Main())

    chassis = BRepPrimAPI_MakeBox(gp_Pnt(-100, -80, 10), 200, 160, 40).Shape()
    # Wheel modelled at the origin, axis +y, centred on its own width.
    wheel = BRepPrimAPI_MakeCylinder(
        gp_Ax2(gp_Pnt(0, -10, 0), gp_Dir(0, 1, 0)), 30, 20).Shape()
    weapon = BRepPrimAPI_MakeCylinder(
        gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 80, 6).Shape()

    asm = tool.NewShape()
    name(asm, "simple_bot")
    products = {}
    for label, shape in (("Chassis", chassis), ("Wheel", wheel), ("Weapon", weapon)):
        products[label] = tool.AddShape(shape, False)
        name(products[label], label)

    for inst, ref, loc in (
        ("chassis", "Chassis", located(0, 0, 0)),
        ("wheel_L", "Wheel", located(0, 100, 30)),
        ("wheel_R", "Wheel", located(0, -100, 30)),
        ("weapon", "Weapon", located(0, 0, 55)),
    ):
        ref = products[ref]
        comp = tool.AddComponent(asm, ref, loc)
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
