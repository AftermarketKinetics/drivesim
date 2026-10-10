//! The sim page's opponent buttons: copy of the robot, orange box, or the
//! AI-driven copy (user, 2026-10-10), and for the AI a public rival robot
//! from the dropdown (`rival_load`). The page calls `set_opponent` and
//! `set_rival`; the `apply_opponent_toggle` system rebuilds the round when
//! the choice differs from what is built.

use crate::Root;
use crate::hud::HudText;
use crate::rival_load::{Rivals, wanted_rival};
use crate::round::{BuiltWith, Loaded, RoundVisuals, build_round};
use bevy::prelude::*;
use drivesim_physics::OpponentKind;
use std::sync::atomic::{AtomicU8, Ordering};
use wasm_bindgen::prelude::wasm_bindgen;

/// The page's latest choice. A static because the page calls in from JS,
/// outside any Bevy system.
static WANT: AtomicU8 = AtomicU8::new(0);

/// Called by index.html's button: 0 the copy, 1 the box, 2 the AI.
#[wasm_bindgen]
pub fn set_opponent(kind: u8) {
    WANT.store(kind, Ordering::Relaxed);
}

/// An opponent as the page chooses it: the kind and, for the AI only,
/// the rival's bundle base URL ("" = the robot's own copy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub kind: OpponentKind,
    pub rival: String,
}

/// The opponent the page last asked for (the copy until it asks, or if
/// it sends a number it shouldn't). The rival is kept only for the AI.
pub fn wanted() -> Choice {
    let kind = match WANT.load(Ordering::Relaxed) {
        1 => OpponentKind::Box,
        2 => OpponentKind::Ai,
        _ => OpponentKind::Copy,
    };
    let rival = match kind {
        OpponentKind::Ai => wanted_rival(),
        _ => String::new(),
    };
    Choice { kind, rival }
}

#[allow(clippy::too_many_arguments)] // a Bevy system: one parameter per resource/query
pub fn apply_opponent_toggle(
    mut commands: Commands,
    loaded: Option<Res<Loaded>>,
    built: Option<Res<BuiltWith>>,
    mut rivals: Option<ResMut<Rivals>>,
    root: Res<Root>,
    old: RoundVisuals,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let (Some(loaded), Some(built)) = (loaded, built) else {
        return;
    };
    let want = wanted();
    if built.0 == want {
        return;
    }
    let rival = match (want.rival.as_str(), rivals.as_mut()) {
        ("", _) => None,
        (_, None) => {
            let e = "no motor library to load it with".to_string();
            return fail(&mut commands, &mut hud, want, e);
        }
        // Still loading: the current round plays on until it arrives.
        (base, Some(r)) => match r.get(base) {
            None => return,
            Some(Err(e)) => return fail(&mut commands, &mut hud, want.clone(), e.clone()),
            Some(Ok(b)) => Some(b),
        },
    };
    if let Err(e) = build_round(
        &mut commands,
        root.0,
        &loaded.0,
        want.clone(),
        rival,
        old.iter(),
        &mut meshes,
        &mut mats,
    ) {
        fail(&mut commands, &mut hud, want, e);
    }
}

/// A switch that can't be made: recorded as built so it isn't retried
/// every frame, with the round in play kept and the HUD saying why.
fn fail(
    commands: &mut Commands,
    hud: &mut Query<&mut Text, With<HudText>>,
    want: Choice,
    e: String,
) {
    commands.insert_resource(BuiltWith(want));
    if let Ok(mut t) = hud.single_mut() {
        **t = format!("could not switch opponent:\n{e}");
    }
}
