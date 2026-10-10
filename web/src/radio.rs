//! The radio through the browser's Gamepad API (docs/drivesim.md §7).
//!
//! Chrome reports an EdgeTX radio in USB joystick mode as a generic pad
//! (`mapping == ""`) whose axis N is channel N+1 with the radio's own sign,
//! verified on the Pocket 2026-10-09. The pad is picked by EdgeTX's USB ID,
//! not by position or name. Bevy's own gamepad support (gilrs) is not used:
//! its guessed layout flips and rescales axes.

use bevy::prelude::*;
use drivesim_input::{Frame, frame_from_axes, is_edgetx_joystick, parse_web_id};
use wasm_bindgen::JsCast;

/// This frame's radio state: `frame` is `None` when no EdgeTX radio is
/// connected (the robot then sees signal loss).
#[derive(Resource, Default)]
pub struct Radio {
    pub frame: Option<Frame>,
    pub name: Option<String>,
}

pub fn poll_radio(mut radio: ResMut<Radio>) {
    let found = web_sys::window()
        .and_then(|w| w.navigator().get_gamepads().ok())
        .and_then(|pads| {
            pads.iter().find_map(|p| {
                let pad: web_sys::Gamepad = p.dyn_into().ok()?;
                let id = parse_web_id(&pad.id())?;
                if !pad.connected() || !is_edgetx_joystick(id) {
                    return None;
                }
                let axes: Vec<f32> = pad
                    .axes()
                    .iter()
                    .map(|a| a.as_f64().unwrap_or(0.0) as f32)
                    .collect();
                Some((pad.id(), axes))
            })
        });
    match found {
        Some((name, axes)) => {
            radio.frame = frame_from_axes(Some(&axes));
            radio.name = Some(name);
        }
        None => {
            radio.frame = None;
            radio.name = None;
        }
    }
}
