# Read the Pocket through gilrs-core, not gilrs

For a pad with no SDL mapping (the Pocket in EdgeTX USB joystick mode reports as a generic "HID-compliant game controller"), gilrs 0.11 guesses a layout from axis order, and two of its guesses corrupt the data. It negates whichever axes it labels LeftStickY/RightStickY: on Windows, `IS_Y_AXIS_REVERSED` is applied in `axis_value`. It also reports the axes it labels triggers as 0..1 instead of −1..+1.

- Seen 2026-10-09: CH2 at +100 read −1 on gilrs `Axis(1)`; axes 3, 4, 6 and 7 read 0..1 with centre 0.5.
- `gilrs-core` 0.6 gives raw `AxisValueChanged(i32, EvCode)` plus each axis's `axis_info` range, with no mapping layer. Normalised by that range, the Pocket's channel N is axis N−1 with the radio's own sign (verified against the channel monitor on all 8 channels).
- gilrs's default filters (deadzone, jitter) also hide small stick moves; a diagnostic tool should not use them.
