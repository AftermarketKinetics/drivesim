# lib.rs (drivesim-input)

## Function

The radio as the sim sees it (docs/drivesim.md §7): EdgeTX joystick axes
in, the CRSF channel frame a receiver would deliver out, so a robot's
control code runs on exactly what it gets on the real robot.

## Interface

Re-exports `channels::{CRSF_MIN, CRSF_MID, CRSF_MAX, Frame,
frame_from_axes, normalize}` and `radio_id::{RadioId, is_edgetx_joystick,
parse_web_id}`.

## Implementation

Pure functions with no dependencies, so the crate builds for the host
(tests) and for wasm32 (the web app) unchanged. Reading the device
(Gamepad API, gilrs-core) belongs to the app.

## Assertions

- No I/O, no platform APIs, no dependencies.
