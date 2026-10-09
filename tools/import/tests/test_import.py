"""Known-answer tests for the STEP importer, on tests/fixtures/simple_bot.step
(built by make_fixture.py: see its header for the geometry)."""

import math
import os
import struct
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))

from step_import import import_step  # noqa: E402

FIXTURE = os.path.join(HERE, "fixtures", "simple_bot.step")


def read_stl(path):
    with open(path, "rb") as f:
        data = f.read()
    n = struct.unpack("<I", data[80:84])[0]
    tris = []
    for i in range(n):
        v = struct.unpack("<9f", data[84 + 50 * i + 12:84 + 50 * i + 48])
        tris.append((v[0:3], v[3:6], v[6:9]))
    return tris


def signed_volume(tris):
    """Positive only when every triangle is wound outward."""
    return sum((a[0] * (b[1] * c[2] - b[2] * c[1])
                - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0])) / 6.0
               for a, b, c in tris)


class ImportFixture(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.doc = import_step(FIXTURE, cls.tmp.name)
        cls.by = {p["instance"]: p for p in cls.doc["parts"]}

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_instances_keep_their_names(self):
        self.assertEqual(sorted(self.by), ["chassis", "weapon", "wheel_L", "wheel_R"])
        self.assertEqual(self.by["wheel_L"]["product"], "Wheel")
        self.assertEqual(self.by["wheel_R"]["path"], "simple_bot/wheel_R")

    def test_volumes_are_exact(self):
        self.assertAlmostEqual(self.by["chassis"]["volume_m3"], 0.2 * 0.16 * 0.04, places=12)
        self.assertAlmostEqual(self.by["wheel_L"]["volume_m3"], math.pi * 0.03**2 * 0.02, places=12)
        self.assertAlmostEqual(self.by["weapon"]["volume_m3"], math.pi * 0.08**2 * 0.006, places=12)

    def test_instances_are_placed(self):
        for name, com in (("chassis", (0, 0, 0.03)), ("wheel_L", (0, 0.1, 0.03)),
                          ("wheel_R", (0, -0.1, 0.03)), ("weapon", (0, 0, 0.058))):
            for got, want in zip(self.by[name]["com_m"], com):
                self.assertAlmostEqual(got, want, places=9, msg=name)

    def test_inertia_is_about_the_centre_at_unit_density(self):
        disc = self.by["weapon"]
        v, r, t = disc["volume_m3"], 0.08, 0.006
        izz = v * r * r / 2
        ixx = v * (3 * r * r + t * t) / 12
        i = disc["inertia_m5"]
        self.assertAlmostEqual(i[2][2] / izz, 1.0, places=9)
        self.assertAlmostEqual(i[0][0] / ixx, 1.0, places=9)
        self.assertAlmostEqual(i[0][1], 0.0, places=15)

    def test_wheels_and_weapon_report_their_axes(self):
        for name, d, r, w in (("wheel_L", (0, 1, 0), 0.03, 0.02),
                              ("weapon", (0, 0, 1), 0.08, 0.006)):
            ax = self.by[name]["rotation_axis"]
            self.assertEqual([round(x, 9) for x in ax["dir"]], list(d), name)
            self.assertAlmostEqual(ax["curved_fraction"] + ax["cap_fraction"], 1.0, places=6)
            self.assertAlmostEqual(ax["com_offset_m"], 0.0, places=9)
            # The mesh's outer radius sits on the true circle (chords lie inside).
            self.assertAlmostEqual(ax["radius_m"], r, delta=1e-6)
            self.assertAlmostEqual(ax["width_m"], w, delta=1e-6)

    def test_a_box_has_no_axis(self):
        self.assertIsNone(self.by["chassis"]["rotation_axis"])

    def test_meshes_are_closed_and_wound_outward(self):
        for p in self.doc["parts"]:
            tris = read_stl(os.path.join(self.tmp.name, p["mesh"]))
            self.assertEqual(len(tris), p["triangles"])
            # Tessellated cylinders lose a little volume to their chords.
            self.assertAlmostEqual(signed_volume(tris) / p["volume_m3"], 1.0,
                                   delta=0.01, msg=p["instance"])


if __name__ == "__main__":
    unittest.main()
