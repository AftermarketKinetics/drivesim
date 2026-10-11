//! The AI bout's clock and its end (user, 2026-10-11: 3-minute bouts,
//! then TIME and a rematch): an m:ss clock at the top centre, and the
//! flags the page's Rematch button reads and sets.

use crate::sim_step::SimRes;
use bevy::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::wasm_bindgen;

/// Whether the bout in play has run out; written each frame, read by the
/// page to show its Rematch button.
static OVER: AtomicBool = AtomicBool::new(false);
/// Set by the page's Rematch button, taken by `opponent_toggle`.
static REMATCH: AtomicBool = AtomicBool::new(false);

/// For the page: true while the bout in play has run out.
#[wasm_bindgen]
pub fn bout_over() -> bool {
    OVER.load(Ordering::Relaxed)
}

/// Called by the page's Rematch button: rebuild the round as it is.
#[wasm_bindgen]
pub fn rematch() {
    REMATCH.store(true, Ordering::Relaxed);
}

/// Whether a rematch was asked for since the last call.
pub fn take_rematch() -> bool {
    REMATCH.swap(false, Ordering::Relaxed)
}

#[derive(Component)]
pub struct BoutClock;

pub fn spawn_bout_clock(commands: &mut Commands) {
    commands.spawn((
        BoutClock,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(40.0),
            ..default()
        },
        TextColor(Color::srgb(0.93, 0.93, 0.93)),
        TextLayout::justify(bevy::text::Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));
}

/// `left` seconds as m:ss, whole seconds rounded up, so the clock reads
/// 0:00 only once time is up.
pub fn clock(left: f32) -> String {
    let s = left.ceil() as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

pub fn update_bout_clock(sim: Option<Res<SimRes>>, mut q: Query<&mut Text, With<BoutClock>>) {
    let left = sim.and_then(|s| s.sim.bout_left());
    OVER.store(left == Some(0.0), Ordering::Relaxed);
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let want = left.map(clock).unwrap_or_default();
    if **text != want {
        **text = want;
    }
}
