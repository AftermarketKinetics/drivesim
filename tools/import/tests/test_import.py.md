# test_import.py

## Function

Known-answer tests for the importer, on the fixture STEP.

## Interface

`python -m unittest discover -s tools/import/tests` (or `make test`).
Tests: `test_instances_keep_their_names`, `test_volumes_are_exact`,
`test_instances_are_placed`,
`test_inertia_is_about_the_centre_at_unit_density`,
`test_wheels_and_weapon_report_their_axes`,
`test_the_toothed_disc_is_balanced_and_holed`,
`test_boxes_and_prisms_have_no_axis`,
`test_meshes_are_closed_and_wound_outward`.

## Implementation

Every fixture part except the weapon is a box, cylinder or straight
triangular prism, so volumes and centroids are closed-form. The toothed,
balance-holed disc is checked by bounds: it is balanced (CoM on its
axis), its radius is the tooth's tip corner, and its volume lies between
the bare disc less a 40 mm hole and the disc plus the whole tooth box. A prism's centroid is its
profile's vertex mean, carried along its extrusion.

The fixture is imported once into a temporary directory per run. The
mesh check uses signed volume: positive and within 1% of the B-rep volume
only if the mesh is closed and every triangle is wound outward.

## Assertions

- Expected values come from the fixture's geometry, never from a previous
  run's output.
