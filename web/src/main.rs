//! drivesim in the browser (docs/drivesim.md §8): fetch a robot bundle,
//! simulate it with drivesim-physics, read the radio through the Gamepad
//! API, and draw it all from the driver's box.
//!
//! `bundle_load` fetches and loads; `radio` reads the pad; `sim_step` runs
//! the physics in real time; `robot_render` and `arena_render` draw;
//! `floor_texture` paints the floor; `camera` is the driver's eye; `hud`
//! is the text overlay.

mod arena_render;
mod bundle_load;
mod camera;
mod floor_texture;
mod hud;
mod radio;
mod robot_render;
mod sim_step;
mod wall_markings;

use bevy::prelude::*;
use bundle_load::{Bundle, fetch_bundle, locate};
use drivesim_physics::Sim;
use robot_render::SimRoot;
use sim_step::SimRes;
use std::sync::{Arc, Mutex};

/// Where the async fetch leaves its result for the app to pick up.
#[derive(Resource, Clone, Default)]
struct LoadSlot(Arc<Mutex<Option<Result<Bundle, String>>>>);

#[derive(Resource)]
struct Root(Entity);

/// The opponent copy is drawn all orange (user, 2026-10-10), so it can't be
/// mistaken for the robot being driven.
const OPPONENT_ORANGE: [f32; 3] = [0.95, 0.45, 0.08];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#drivesim".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.07, 0.07, 0.08)))
        .init_resource::<radio::Radio>()
        .init_resource::<LoadSlot>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                radio::poll_radio,
                take_bundle,
                sim_step::step_sim,
                robot_render::sync_bodies,
                hud::update_hud,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    slot: Res<LoadSlot>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Rapier's world is z-up; Bevy's is y-up. Everything simulated hangs
    // off this one rotation.
    let root = commands
        .spawn((
            SimRoot,
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Visibility::default(),
        ))
        .id();
    commands.insert_resource(Root(root));
    arena_render::spawn_arena(&mut commands, root, &mut meshes, &mut mats, &mut images);
    camera::spawn_camera(&mut commands, root);
    hud::spawn_hud(&mut commands);

    let slot = slot.0.clone();
    match locate() {
        Ok((base, motors)) => wasm_bindgen_futures::spawn_local(async move {
            let result = fetch_bundle(base, motors).await;
            *slot.lock().unwrap() = Some(result);
        }),
        Err(e) => *slot.lock().unwrap() = Some(Err(e)),
    }
}

/// When the fetch finishes: build the simulation and its visuals, or show
/// why it failed.
fn take_bundle(
    mut commands: Commands,
    slot: Res<LoadSlot>,
    root: Res<Root>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut hud: Query<&mut Text, With<hud::HudText>>,
) {
    let Some(result) = slot.0.lock().unwrap().take() else {
        return;
    };
    let fail = |hud: &mut Query<&mut Text, With<hud::HudText>>, msg: String| {
        if let Ok(mut t) = hud.single_mut() {
            **t = format!("could not load the robot:\n{msg}");
        }
    };
    let bundle = match result {
        Ok(b) => b,
        Err(e) => return fail(&mut hud, e),
    };
    let sim = match Sim::new(bundle.model.clone(), &|p| bundle.meshes.get(p).cloned()) {
        Ok(s) => SimRes::new(s),
        Err(e) => return fail(&mut hud, e.to_string()),
    };
    let s = &sim.sim;
    robot_render::spawn_robot_visuals(
        &mut commands,
        root.0,
        &bundle,
        &s.model,
        &s.robot,
        None,
        &mut meshes,
        &mut mats,
    );
    if let Some(o) = &s.opponent {
        robot_render::spawn_robot_visuals(
            &mut commands,
            root.0,
            &bundle,
            &s.model,
            &o.robot,
            Some(OPPONENT_ORANGE),
            &mut meshes,
            &mut mats,
        );
    }
    commands.insert_resource(sim);
}
