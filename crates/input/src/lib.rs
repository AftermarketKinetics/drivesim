//! The radio as the sim sees it (docs/drivesim.md §7): an EdgeTX radio in
//! USB joystick mode (Classic) reports channel N as axis N-1 with the
//! radio's own sign, on Windows through gilrs-core and in Chrome through
//! the Gamepad API (both verified 2026-10-09 on a RadioMaster Pocket).
//! This crate turns those axes into the CRSF channel frame a robot's
//! control receives, so the sim's control sees exactly what a receiver
//! would. Reading the device is the app's job; nothing here touches it.

pub mod channels;
pub mod radio_id;

pub use channels::{CRSF_MAX, CRSF_MID, CRSF_MIN, Frame, frame_from_axes, normalize};
pub use radio_id::{RadioId, is_edgetx_joystick, parse_web_id};
