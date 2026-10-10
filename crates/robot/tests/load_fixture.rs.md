# load_fixture.rs

## Function

Tests `load` end to end on the fixture bundle `fixtures/simple_bot`.

## Interface

Tests: `masses_come_from_densities_and_overrides`,
`the_wheel_has_a_cylinders_inertia`,
`wheels_and_weapon_get_their_axes`,
`total_mass_scales_every_part_and_keeps_the_com`,
`mass_override_replaces_density`, `a_y_up_step_is_turned_upright`,
`mistakes_are_named_errors`.

## Implementation

Error cases edit the fixture's robot.toml text with one `replacen`; the
helper asserts the text being replaced exists, so a fixture edit cannot
silently turn a test into a no-op.

Since the fixture became a vertical spinner (2026-10-10), the chassis
is 11 parts, so its exact check is a mass (density sums plus the
overridden electronics), not a box's inertia. The disc keeps the
closed-form inertia check. Since the disc got its tooth and balance
hole (2026-10-10), it has no closed form either: its mass is checked as
density × the importer's measured volume, and the closed-form inertia
check moved to a wheel.

## Assertions

- Expected numbers come from the fixture geometry (make_fixture.py) and
  the densities, never from a previous run.
