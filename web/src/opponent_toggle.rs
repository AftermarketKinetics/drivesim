//! The sim page's opponent button: copy of the robot or orange box
//! (user, 2026-10-10). The page calls `set_opponent_box`; the
//! `apply_opponent_toggle` system rebuilds the round when the choice
//! differs from what is built.

use crate::Root;
use crate::hud::HudText;
use crate::round::{BuiltWith, Loaded, RoundVisuals, build_round};
use bevy::prelude::*;
use drivesim_physics::OpponentKind;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::wasm_bindgen;

/// The page's latest choice. A static because the page calls in from JS,
/// outside any Bevy system.
static WANT_BOX: AtomicBool = AtomicBool::new(false);

/// Called by index.html's button: `true` for the box, `false` for the copy.
#[wasm_bindgen]
pub fn set_opponent_box(on: bool) {
    WANT_BOX.store(on, Ordering::Relaxed);
}

/// The opponent the page last asked for (the copy until it asks).
pub fn wanted() -> OpponentKind {
    if WANT_BOX.load(Ordering::Relaxed) {
        OpponentKind::Box
    } else {
        OpponentKind::Copy
    }
}

#[allow(clippy::too_many_arguments)] // a Bevy system: one parameter per resource/query
pub fn apply_opponent_toggle(
    mut commands: Commands,
    loaded: Option<Res<Loaded>>,
    built: Option<Res<BuiltWith>>,
    root: Res<Root>,
    old: RoundVisuals,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let (Some(loaded), Some(built)) = (loaded, built) else {
        return;
    };
    let kind = wanted();
    if built.0 == kind {
        return;
    }
    if let Err(e) = build_round(
        &mut commands,
        root.0,
        &loaded.0,
        kind,
        old.iter(),
        &mut meshes,
        &mut mats,
    ) {
        // The same model built a moment ago, so this should not happen;
        // stop retrying every frame and say why.
        commands.insert_resource(BuiltWith(kind));
        if let Ok(mut t) = hud.single_mut() {
            **t = format!("could not switch opponent:\n{e}");
        }
    }
}
