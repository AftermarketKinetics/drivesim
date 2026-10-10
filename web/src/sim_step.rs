//! Runs the simulation in real time: control ticks at `CONTROL_HZ`, each
//! running `STEPS_PER_TICK` physics steps (docs/drivesim.md §6).

use crate::radio::Radio;
use bevy::prelude::*;
use drivesim_physics::{CONTROL_HZ, Sim};

/// Longest frame time made up for, s. A slower frame (a tab in the
/// background, a GC pause) drops the rest rather than running a burst of
/// physics that would freeze the next frame too.
pub const MAX_CATCH_UP_S: f32 = 0.05;

/// Smoothing of the headroom readout: each frame moves it this far.
const HEADROOM_EMA: f64 = 0.02;

#[derive(Resource)]
pub struct SimRes {
    pub sim: Sim,
    behind_s: f32,
    /// Simulated seconds per wall-clock second spent stepping (smoothed):
    /// how many times faster than real time the physics could run. Below
    /// 1 the sim cannot keep up.
    pub headroom: f64,
}

impl SimRes {
    pub fn new(sim: Sim) -> Self {
        SimRes {
            sim,
            behind_s: 0.0,
            headroom: 0.0,
        }
    }
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now())
}

pub fn step_sim(time: Res<Time>, radio: Res<Radio>, s: Option<ResMut<SimRes>>) {
    let Some(mut s) = s else { return };
    let tick = 1.0 / CONTROL_HZ as f32;
    s.behind_s += time.delta_secs().min(MAX_CATCH_UP_S);
    let (start, mut ticks) = (now_ms(), 0u32);
    while s.behind_s >= tick {
        s.sim.control_tick(radio.frame.as_ref());
        s.behind_s -= tick;
        ticks += 1;
    }
    let spent_ms = now_ms() - start;
    if ticks > 0 && spent_ms > 0.0 {
        let h = ticks as f64 * tick as f64 * 1e3 / spent_ms;
        s.headroom = if s.headroom == 0.0 {
            h
        } else {
            s.headroom + HEADROOM_EMA * (h - s.headroom)
        };
    }
}
