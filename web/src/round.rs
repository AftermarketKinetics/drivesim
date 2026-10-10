//! A round: the simulation built from a loaded bundle, and its visuals.
//! Built once when the bundle arrives and again whenever the driver
//! switches opponent (or the AI's rival robot), which puts both back at
//! their start spots.

use crate::bundle_load::Bundle;
use crate::opponent_toggle::Choice;
use crate::robot_render::{self, BodyLink};
use crate::sim_step::SimRes;
use crate::target_render::spawn_target_visual;
use bevy::prelude::*;
use drivesim_physics::{Rival, Sim};

/// The opponent is drawn all orange (user, 2026-10-10), copy, rival or
/// box, so it can't be mistaken for the robot being driven.
pub const OPPONENT_ORANGE: [f32; 3] = [0.95, 0.45, 0.08];

/// The loaded bundle, kept so a round can be rebuilt without refetching.
#[derive(Resource)]
pub struct Loaded(pub Bundle);

/// Which opponent the current round was built with.
#[derive(Resource)]
pub struct BuiltWith(pub Choice);

/// Despawns the previous round's visuals (everything linked to a body)
/// and inserts a fresh `SimRes` with `choice`'s opponent and its visuals.
/// `rival` is the loaded bundle of `choice.rival`, `None` for the copy.
#[allow(clippy::too_many_arguments)] // Bevy assets travel separately
pub fn build_round(
    commands: &mut Commands,
    root: Entity,
    bundle: &Bundle,
    choice: Choice,
    rival: Option<&Bundle>,
    old: impl Iterator<Item = Entity>,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) -> Result<(), String> {
    let rival_mesh = |p: &str| rival.and_then(|r| r.meshes.get(p).cloned());
    let sim = Sim::versus(
        bundle.model.clone(),
        &|p| bundle.meshes.get(p).cloned(),
        choice.kind,
        rival.map(|r| Rival {
            model: r.model.clone(),
            mesh: &rival_mesh,
        }),
    )
    .map_err(|e| e.to_string())?;
    for e in old {
        commands.entity(e).despawn();
    }
    robot_render::spawn_robot_visuals(
        commands, root, bundle, &sim.model, &sim.robot, None, meshes, mats,
    );
    if let Some(o) = &sim.opponent {
        robot_render::spawn_robot_visuals(
            commands,
            root,
            rival.unwrap_or(bundle),
            &o.model,
            &o.robot,
            Some(OPPONENT_ORANGE),
            meshes,
            mats,
        );
    }
    if let Some(t) = sim.target {
        spawn_target_visual(commands, root, t, OPPONENT_ORANGE, meshes, mats);
    }
    commands.insert_resource(SimRes::new(sim));
    commands.insert_resource(BuiltWith(choice));
    Ok(())
}

/// The visuals of the round in play, for `build_round` to replace.
pub type RoundVisuals<'w, 's> = Query<'w, 's, Entity, With<BodyLink>>;
