# load_fixture.rs

## Function

Tests `load` end to end on the fixture bundle `fixtures/simple_bot`.

## Interface

Tests: `masses_come_from_densities`, `chassis_inertia_is_the_boxs`,
`wheels_and_weapon_get_their_axes`,
`total_mass_scales_every_part_and_keeps_the_com`,
`mass_override_replaces_density`, `a_y_up_step_is_turned_upright`,
`mistakes_are_named_errors`.

## Implementation

Error cases edit the fixture's robot.toml text with one `replacen`; the
helper asserts the text being replaced exists, so a fixture edit cannot
silently turn a test into a no-op.

## Assertions

- Expected numbers come from the fixture geometry (make_fixture.py) and
  the densities, never from a previous run.
