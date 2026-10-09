# test_import.py

## Function

Known-answer tests for the importer, on the fixture STEP.

## Interface

`python -m unittest discover -s tools/import/tests` (or `make test`).
Tests: `test_instances_keep_their_names`, `test_volumes_are_exact`,
`test_instances_are_placed`,
`test_inertia_is_about_the_centre_at_unit_density`,
`test_wheels_and_weapon_report_their_axes`, `test_a_box_has_no_axis`,
`test_meshes_are_closed_and_wound_outward`.

## Implementation

The fixture is imported once into a temporary directory per run. The
mesh check uses signed volume: positive and within 1% of the B-rep volume
only if the mesh is closed and every triangle is wound outward.

## Assertions

- Expected values come from the fixture's geometry, never from a previous
  run's output.
