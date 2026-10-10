//! drivesim in the browser (docs/drivesim.md §8): fetch a robot bundle,
//! simulate it with drivesim-physics, read the radio through the Gamepad
//! API, and draw it all from the driver's box.
//!
//! `bundle_load` fetches and loads; `round` builds the simulation and its
//! visuals, again whenever `opponent_toggle` (the page's button) switches
//! opponent, with `rival_load` fetching the AI's rival robot; `radio`
//! reads the pad; `sim_step` runs the physics in real
//! time; `robot_render`, `target_render` and `arena_render` draw;
//! `floor_texture` paints the floor; `camera` is the driver's eye; `hud`
//! is the text overlay, and `countdown` the AI round's 3-2-1.

mod arena_render;
mod bundle_load;
mod camera;
mod countdown;
mod floor_texture;
mod hud;
mod opponent_toggle;
mod radio;
mod rival_load;
mod robot_render;
mod round;
mod sim_step;
mod target_render;
mod wall_markings;

use bevy::prelude::*;
use bundle_load::{Bundle, fetch_bundle, locate};
use robot_render::SimRoot;
use round::{Loaded, RoundVisuals, build_round};
use std::sync::{Arc, Mutex};

/// Where the async fetch leaves its result for the app to pick up.
#[derive(Resource, Clone, Default)]
struct LoadSlot(Arc<Mutex<Option<Result<Bundle, String>>>>);

#[derive(Resource)]
struct Root(Entity);

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
                opponent_toggle::apply_opponent_toggle,
                sim_step::step_sim,
                robot_render::sync_bodies,
                hud::update_hud,
                countdown::update_countdown,
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
    countdown::spawn_countdown(&mut commands);

    let slot = slot.0.clone();
    match locate() {
        Ok((base, motors)) => {
            commands.insert_resource(rival_load::Rivals::new(motors.clone()));
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_bundle(base, motors).await;
                *slot.lock().unwrap() = Some(result);
            })
        }
        Err(e) => *slot.lock().unwrap() = Some(Err(e)),
    }
}

/// When the fetch finishes: build the first round, with the opponent the
/// page asks for (as the copy, if it wants a rival: `apply_opponent_toggle`
/// swaps that in once it loads), or show why it failed.
fn take_bundle(
    mut commands: Commands,
    slot: Res<LoadSlot>,
    root: Res<Root>,
    old: RoundVisuals,
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
    if let Err(e) = build_round(
        &mut commands,
        root.0,
        &bundle,
        opponent_toggle::Choice {
            kind: opponent_toggle::wanted().kind,
            rival: String::new(),
        },
        None,
        old.iter(),
        &mut meshes,
        &mut mats,
    ) {
        return fail(&mut hud, e);
    }
    commands.insert_resource(Loaded(bundle));
}
