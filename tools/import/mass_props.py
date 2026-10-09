"""Exact unit-density mass properties of one solid, in SI units."""

from OCC.Core.BRepGProp import brepgprop
from OCC.Core.GProp import GProp_GProps

MM = 1e-3


def mass_props(solid):
    """Volume (m^3), centre of volume (m), and the inertia tensor about that
    centre at a density of 1 kg/m^3 (kg m^2, i.e. m^5), all in the assembly
    frame. Multiply the volume and the tensor by a density to get mass and
    inertia.

    OCC integrates the B-rep exactly; its MatrixOfInertia is already about
    the centre of mass, in global axes. Input is in mm (OCC's STEP unit).
    """
    g = GProp_GProps()
    brepgprop.VolumeProperties(solid, g)
    c = g.CentreOfMass()
    m = g.MatrixOfInertia()
    return {
        "volume_m3": g.Mass() * MM**3,
        "com_m": [c.X() * MM, c.Y() * MM, c.Z() * MM],
        "inertia_m5": [[m.Value(i, j) * MM**5 for j in (1, 2, 3)]
                       for i in (1, 2, 3)],
    }
