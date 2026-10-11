//! Glowing impact marks (user, 2026-10-11): where a robot takes a large
//! impact (`drivesim_physics::impacts`), a spot glows orange for a
//! moderate hit, deep red and bigger for a hard one, and keeps glowing
//! for the rest of the round (user, 2026-10-11: "make them not fade at
//! all"). The camera's bloom (`camera.rs`) gives them a halo.

use crate::robot_render::BodyLink;
use crate::sim_step::SimRes;
use bevy::prelude::*;
use drivesim_physics::impacts::{Impact, MARK_DV};

/// A hit this many times `MARK_DV` or harder is fully red and largest.
const RED_AT: f32 = 5.0;
/// The most marks kept per round; the oldest go first.
pub const MAX_MARKS: usize = 200;
/// Mark radius for a weakest and a hardest hit on a 12 lb robot, m; other
/// robots scale by their size (mass to the one-third power).
const R_MIN_M: f32 = 0.008;
const R_MAX_M: f32 = 0.025;
const MASS_12LB_KG: f32 = 5.44;
/// A new impact this close to a mark under `MERGE_S` old, on the same
/// body, is the same hit continuing over several steps: it heats that
/// mark rather than adding one. In radii of the existing mark.
const MERGE_R: f32 = 1.5;
const MERGE_S: f32 = 0.15;

const ORANGE: [f32; 3] = [1.0, 0.5, 0.08];
const RED: [f32; 3] = [0.85, 0.05, 0.02];
/// Glow brightness over the base colour: well past 1, so the camera's
/// bloom spreads it (user, 2026-10-11: "glow more"; was 6).
const GLOW: f32 = 40.0;

#[derive(Component)]
pub struct Mark {
    body: Entity,
    point: Vec3,
    /// 0 for the weakest marked hit, 1 for `RED_AT` and harder.
    heat: f32,
    radius: f32,
    born_s: f32,
}

/// The shared sphere every mark is drawn with, scaled per mark.
#[derive(Resource)]
pub struct MarkMesh(Handle<Mesh>);

pub fn setup_marks(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(MarkMesh(
        meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
    ));
}

/// 0..1 from `MARK_DV` to `RED_AT` times it, on a log scale: hits span
/// about 2 to 12 m/s, and each doubling should look a step hotter.
pub fn heat(dv: f32) -> f32 {
    ((dv / MARK_DV).ln() / RED_AT.ln()).clamp(0.0, 1.0)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

/// A mark's colour and glow for its heat.
fn look(heat: f32) -> (Color, LinearRgba) {
    let [r, g, b] = mix(ORANGE, RED, heat);
    (
        Color::srgb(r, g, b),
        LinearRgba::rgb(r * GLOW, g * GLOW, b * GLOW),
    )
}

/// Takes the sim's new impacts and adds or reheats marks on the bodies
/// hit, keeping at most `MAX_MARKS`.
pub fn update_marks(
    mut commands: Commands,
    sim: Option<ResMut<SimRes>>,
    mesh: Res<MarkMesh>,
    bodies: Query<(Entity, &BodyLink)>,
    mut marks: Query<(
        Entity,
        &mut Mark,
        &MeshMaterial3d<StandardMaterial>,
        &mut Transform,
    )>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut sim) = sim else { return };
    let now = sim.sim.time_s;
    let impacts = sim.sim.take_impacts();
    let scale = |i: &Impact| {
        let s = &sim.sim;
        let mass = match &s.opponent {
            Some(o) if o.robot.bodies.contains(&i.body) => o.model.total_mass(),
            _ => s.model.total_mass(),
        } as f32;
        (mass / MASS_12LB_KG).cbrt()
    };
    // Marks to spawn this frame. A frame runs dozens of physics steps, so
    // one hit's steps also merge among themselves here.
    let mut new: Vec<Mark> = Vec::new();
    for i in &impacts {
        let Some((body, _)) = bodies.iter().find(|(_, l)| l.0 == i.body) else {
            continue;
        };
        let point = Vec3::new(i.point.x, i.point.y, i.point.z);
        let h = heat(i.dv);
        let radius = (R_MIN_M + (R_MAX_M - R_MIN_M) * h) * scale(i);
        let near = marks.iter_mut().find(|(_, m, _, _)| {
            m.body == body
                && now - m.born_s < MERGE_S
                && m.point.distance(point) < MERGE_R * m.radius
        });
        if let Some((_, mut m, mat, mut t)) = near {
            m.heat = m.heat.max(h);
            m.radius = m.radius.max(radius);
            t.scale = Vec3::splat(m.radius);
            if let Some(mut mat) = mats.get_mut(&mat.0) {
                (mat.base_color, mat.emissive) = look(m.heat);
            }
            continue;
        }
        let pending = new
            .iter_mut()
            .find(|m| m.body == body && m.point.distance(point) < MERGE_R * m.radius);
        match pending {
            Some(m) => {
                m.heat = m.heat.max(h);
                m.radius = m.radius.max(radius);
            }
            None => new.push(Mark {
                body,
                point,
                heat: h,
                radius,
                born_s: now,
            }),
        }
    }
    for m in new {
        let (base, emissive) = look(m.heat);
        let (body, point, radius) = (m.body, m.point, m.radius);
        let mark = commands
            .spawn((
                m,
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(mats.add(StandardMaterial {
                    base_color: base,
                    emissive,
                    perceptual_roughness: 0.9,
                    ..default()
                })),
                Transform::from_translation(point).with_scale(Vec3::splat(radius)),
            ))
            .id();
        commands.entity(body).add_child(mark);
    }
    let mut live: Vec<(f32, Entity)> = marks.iter().map(|(e, m, _, _)| (m.born_s, e)).collect();
    if live.len() > MAX_MARKS {
        live.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, e) in &live[..live.len() - MAX_MARKS] {
            commands.entity(*e).despawn();
        }
    }
}
