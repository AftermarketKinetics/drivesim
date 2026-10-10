# radio.rs

## Function

Reads the radio through the browser's Gamepad API, once a frame
(docs/drivesim.md §7).

## Interface

- `Radio { frame: Option<Frame>, name: Option<String> }`: a resource.
  `frame` is `None` when no EdgeTX radio is connected, and the robot then
  sees signal loss.
- `poll_radio`: a system that refreshes `Radio`.

## Implementation

- **The pad is found by EdgeTX's USB ID** (VID 1209, PID 4F54, read from
  Chrome's id string by `parse_web_id`), not by index or name.
- **Chrome reports the radio as a generic pad** (`mapping == ""`). Its
  axis N is channel N+1 with the radio's own sign; this was verified on
  the Pocket on 2026-10-09 with `tools/padtest`. `frame_from_axes` does
  the channel mapping.
- **Bevy's gamepad support (gilrs) is not compiled in.** Its guessed
  layout flips stick-Y axes and rescales others (lesson
  `gilrs-guesses-unknown-pads.md`).
- Chrome exposes no pad until the user presses a button or moves an axis
  on the page. That is why the HUD says to move a stick.

## Assertions

- Only an EdgeTX joystick is ever read; any other pad is ignored.
- Axis values reach `frame_from_axes` unscaled and unflipped.
