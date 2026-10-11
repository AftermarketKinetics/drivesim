//! The physics rate the page's slider asks for (user, 2026-10-11: 2 to
//! 4 kHz, 2 by default), applied to whichever round is running.

use crate::sim_step::SimRes;
use bevy::prelude::*;
use drivesim_physics::PHYSICS_HZ;
use std::sync::atomic::{AtomicU32, Ordering};
use wasm_bindgen::prelude::wasm_bindgen;

/// The rate the page last asked for, Hz. A static because the slider
/// calls in from JS.
static WANTED_HZ: AtomicU32 = AtomicU32::new(PHYSICS_HZ);

/// Called by the page's "Physics" slider, in kHz. `Sim::set_physics_hz`
/// takes it to the nearest rate it offers.
#[wasm_bindgen]
pub fn set_physics_khz(khz: u32) {
    WANTED_HZ.store(khz * 1000, Ordering::Relaxed);
}

/// Before `step_sim`: puts the running round, and every new one, on the
/// wanted rate.
pub fn apply_physics_rate(s: Option<ResMut<SimRes>>) {
    let Some(mut s) = s else { return };
    let hz = WANTED_HZ.load(Ordering::Relaxed);
    if s.sim.physics_hz() != hz {
        s.sim.set_physics_hz(hz);
    }
}
