"""Reads a STEP assembly into located leaf solids, with names and colours.

Adapted from the cad-step plugin's `occshapes.load_parts`
(~/code/cad-step-parser-skills, 2026-10-09), cut down to what the importer
needs: no STEP entity recovery, and instance and product names kept apart,
because the roles skill matches on both.
"""

from dataclasses import dataclass

from OCC.Core.IFSelect import IFSelect_RetDone
from OCC.Core.Quantity import Quantity_Color, Quantity_TOC_RGB
from OCC.Core.STEPCAFControl import STEPCAFControl_Reader
from OCC.Core.TDF import TDF_Label, TDF_LabelSequence
from OCC.Core.TDocStd import TDocStd_Document
from OCC.Core.TopAbs import TopAbs_SOLID
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.TopLoc import TopLoc_Location
from OCC.Core.XCAFDoc import (XCAFDoc_ColorCurv, XCAFDoc_ColorGen,
                              XCAFDoc_ColorSurf, XCAFDoc_DocumentTool)

# XCAF's names for shapes the exporter never labelled.
_PLACEHOLDER = ("COMPOUND", "SOLID", "SHELL", "UNNAMED", "")


@dataclass
class Leaf:
    """One solid, located in the assembly frame (mm, as OCC reads STEP)."""
    instance: str      # nearest real instance name up the tree
    product: str       # the product (part definition) it instantiates
    path: str          # instance names from the root, "/"-joined
    shape: object      # TopoDS_Solid, already located
    color: tuple | None


def real_name(name):
    if not name or name.upper() in _PLACEHOLDER:
        return False
    return not name.startswith("=>") and not name.startswith("[0:")


def load(path):
    """Every solid in the file as a `Leaf`, in tree order.

    A product holding several solids yields one leaf per solid, with
    `#n` appended to the instance name from the second on.
    """
    # TDocStd_Document must get a plain str: an ExtendedString aborts the
    # interpreter (OCCT 7.9, noted in the cad-step plugin).
    doc = TDocStd_Document("drivesim-import")
    shapes = XCAFDoc_DocumentTool.ShapeTool(doc.Main())
    colors = XCAFDoc_DocumentTool.ColorTool(doc.Main())

    reader = STEPCAFControl_Reader()
    reader.SetColorMode(True)
    reader.SetNameMode(True)
    if reader.ReadFile(path) != IFSelect_RetDone:
        raise IOError("could not read STEP file: %s" % path)
    reader.Transfer(doc)

    def color_of(label, shape):
        c = Quantity_Color(0.6, 0.6, 0.6, Quantity_TOC_RGB)
        for kind in (XCAFDoc_ColorSurf, XCAFDoc_ColorGen, XCAFDoc_ColorCurv):
            for get in (lambda: colors.GetInstanceColor(shape, kind, c),
                        lambda: colors.GetColor(label, kind, c)):
                try:
                    if get():
                        return (c.Red(), c.Green(), c.Blue())
                except Exception:
                    pass
        return None

    leaves = []

    def recurse(label, loc, trail, instance):
        if shapes.IsAssembly(label):
            comps = TDF_LabelSequence()
            shapes.GetComponents(label, comps)
            for i in range(1, comps.Length() + 1):
                child = comps.Value(i)
                if not shapes.IsReference(child):
                    recurse(child, loc, trail, instance)
                    continue
                referred = TDF_Label()
                shapes.GetReferredShape(child, referred)
                cname = child.GetLabelName()
                if not real_name(cname):
                    cname = referred.GetLabelName()
                recurse(referred, loc.Multiplied(shapes.GetLocation(child)),
                        trail + [cname], cname)
            return
        shape = shapes.GetShape(label)
        if shape is None or shape.IsNull():
            return
        product = label.GetLabelName()
        color = color_of(label, shape)
        placed = shape if loc.IsIdentity() else shape.Located(loc)
        name = instance if real_name(instance) else product
        exp = TopExp_Explorer(placed, TopAbs_SOLID)
        n = 0
        while exp.More():
            suffix = "" if n == 0 else "#%d" % n
            leaves.append(Leaf(name + suffix, product,
                               "/".join(trail) + suffix, exp.Current(), color))
            n += 1
            exp.Next()

    roots = TDF_LabelSequence()
    shapes.GetFreeShapes(roots)
    for i in range(1, roots.Length() + 1):
        root = roots.Value(i)
        rname = root.GetLabelName()
        recurse(root, TopLoc_Location(), [rname], rname)
    return leaves
