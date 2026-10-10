# check_fixture.rs

## Function

Runs the built `drivesim-check` binary the way the server will, on the
fixture bundle and on a broken one.

## Interface

Tests: `the_fixture_reports_its_numbers` (5.265 kg; settled; the disc
off the floor),
`a_bad_bundle_is_a_json_error_not_a_crash`.

## Implementation

The binary comes from `CARGO_BIN_EXE_drivesim-check`. The broken bundle
is the fixture's `parts.json` with an invalid `robot.toml`, in a temp
directory.

## Assertions

- Checks exit codes and JSON, not human text.
