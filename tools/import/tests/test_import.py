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
        self.assertEqual(sorted(self.by), sorted([
            "base_plate", "rail_L", "rail_R", "top_plate", "rear_bar", "battery", "esc",
            "fork_L", "fork_R", "wheel_L", "wheel_R", "drive_motor_L", "drive_motor_R",
            "weapon"]))
        self.assertEqual(self.by["wheel_L"]["product"], "Wheel")
        self.assertEqual(self.by["fork_R"]["product"], "Fork")
        self.assertEqual(self.by["wheel_R"]["path"], "simple_bot/wheel_R")

    def test_volumes_are_exact(self):
        self.assertAlmostEqual(self.by["base_plate"]["volume_m3"], 0.25 * 0.4 * 0.003, places=12)
        self.assertAlmostEqual(self.by["wheel_L"]["volume_m3"],
                               math.pi * 0.03175**2 * 0.025, places=12)
        # The fork is a right-triangle prism: legs 80 and 38 mm, 25 mm wide.
        self.assertAlmostEqual(self.by["fork_L"]["volume_m3"], 0.5 * 0.08 * 0.038 * 0.025,
                               places=12)

    def test_instances_are_placed(self):
        for name, com in (("base_plate", (-0.025, 0, 0.0105)),
                          ("wheel_L", (-0.03, 0.22, 0.03175)),
                          ("wheel_R", (-0.03, -0.22, 0.03175)),
                          # Balanced (make_fixture.py sizes its hole).
                          ("weapon", (0.15, 0, 0.095)),
                          # A triangle's centroid is its vertices' mean.
                          ("fork_L", ((0.12 + 0.2 + 0.12) / 3, 0.0425, (0.002 * 2 + 0.04) / 3)),
                          ("fork_R", ((0.12 + 0.2 + 0.12) / 3, -0.0425, (0.002 * 2 + 0.04) / 3))):
            for got, want in zip(self.by[name]["com_m"], com):
                self.assertAlmostEqual(got, want, places=9, msg=name)

    def test_inertia_is_about_the_centre_at_unit_density(self):
        wheel = self.by["wheel_L"]
        v, r, t = wheel["volume_m3"], 0.03175, 0.025
        i_axis = v * r * r / 2
        i_across = v * (3 * r * r + t * t) / 12
        i = wheel["inertia_m5"]
        self.assertAlmostEqual(i[1][1] / i_axis, 1.0, places=9)
        self.assertAlmostEqual(i[0][0] / i_across, 1.0, places=9)
        self.assertAlmostEqual(i[0][1], 0.0, places=15)

    def test_wheels_and_weapon_report_their_axes(self):
        # The weapon's radius is its tooth's tip corner, not the disc's rim.
        for name, d, r, w in (("wheel_L", (0, 1, 0), 0.03175, 0.025),
                              ("weapon", (0, 1, 0), math.hypot(0.09, 0.015), 0.01)):
            ax = self.by[name]["rotation_axis"]
            self.assertEqual([round(x, 9) for x in ax["dir"]], list(d), name)
            # Only a pure solid of revolution is all curved faces and caps;
            # the tooth's sides are flat but not square to the axis.
            if name == "wheel_L":
                self.assertAlmostEqual(ax["curved_fraction"] + ax["cap_fraction"], 1.0, places=6)
            else:
                self.assertLess(ax["curved_fraction"] + ax["cap_fraction"], 1.0)
            self.assertAlmostEqual(ax["com_offset_m"], 0.0, places=9)
            # The mesh's outer radius sits on the true circle (chords lie inside).
            self.assertAlmostEqual(ax["radius_m"], r, delta=1e-6)
            self.assertAlmostEqual(ax["width_m"], w, delta=1e-6)

    def test_the_toothed_disc_is_balanced_and_holed(self):
        w = self.by["weapon"]
        self.assertLess(w["rotation_axis"]["com_offset_m"], 1e-6)
        # Disc plus the tooth's outboard part, less the balance hole: it
        # lies between the bare disc less a 40 mm hole and the disc plus
        # the whole tooth box.
        disc = math.pi * 0.07**2 * 0.01
        self.assertGreater(w["volume_m3"], disc - math.pi * 0.02**2 * 0.01)
        self.assertLess(w["volume_m3"], disc + 0.03 * 0.03 * 0.01)

    def test_boxes_and_prisms_have_no_axis(self):
        for name in ("base_plate", "rail_L", "fork_L", "battery"):
            self.assertIsNone(self.by[name]["rotation_axis"], name)

    def test_meshes_are_closed_and_wound_outward(self):
        for p in self.doc["parts"]:
            tris = read_stl(os.path.join(self.tmp.name, p["mesh"]))
            self.assertEqual(len(tris), p["triangles"])
            # Tessellated cylinders lose a little volume to their chords.
            self.assertAlmostEqual(signed_volume(tris) / p["volume_m3"], 1.0,
                                   delta=0.01, msg=p["instance"])


if __name__ == "__main__":
    unittest.main()
