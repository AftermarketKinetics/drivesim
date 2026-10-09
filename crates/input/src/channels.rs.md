# channels.rs

## Function

Turns the radio's joystick axes (−1..+1) into one 16-channel CRSF frame:
channel N is axis N−1, as verified on the Pocket in Windows and Chrome on
2026-10-09.

## Interface

- `CRSF_MIN` 172, `CRSF_MID` 992, `CRSF_MAX` 1811.
- `type Frame = [u16; 16]`.
- `frame_from_axes(axes: Option<&[f32]>) -> Option<Frame>`: `None` in
  (no radio) gives `None` out, which a control reads as signal loss.
- `normalize(raw) -> f32`: a channel value back to −1..+1 (momentum-core's
  `crsf::normalize`), for controls that read the frame.

## Implementation

- Each half of the range is scaled separately, the exact inverse of
  momentum-core's `crsf::normalize` (820 counts below centre, 819 above),
  so ±1 and 0 land exactly on the three constants.
- Missing axes and channels 9–16 sit at centre; extra axes past 16 are
  ignored; out-of-range input clamps; NaN reads as centre.

## Assertions

- The three constants equal momentum-core's `crsf` constants; a change
  there must be mirrored here (to be checked by a test once the
  momentum-core control hook depends on both).
- Tested: `normalize` inverts `frame_from_axes` to within one count; ends and centre, axis-to-channel order, half stick, clamping
  and NaN, no radio, more than 16 axes.
