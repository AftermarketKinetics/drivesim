//! Distance markings on the cage walls (user, 2026-10-10): 1 m black and
//! white squares along the bottom of the far and side walls, and a ruler
//! (ticks every 0.1 m, longer at 0.5 m and 1 m) along the base of all
//! four. Both count from each wall's left-hand corner seen from inside the
//! cage, so a square's edge is a whole metre from a corner.

use bevy::prelude::*;
use drivesim_physics::arena::ARENA_SIDE_M;

/// Squares, m: one metre on a side, one row from the floor up.
pub const SQUARE_M: f32 = 1.0;
/// Ruler tick spacing and heights, m.
const TICK_PITCH_M: f32 = 0.1;
const TICK_H: [f32; 3] = [0.04, 0.08, 0.15]; // 0.1 m, 0.5 m, 1 m
const TICK_W_M: f32 = 0.006;
/// How far in front of the wall's inner face the markings sit, m, so they
/// never z-fight it (ticks a little further, in front of the squares).
const SQUARE_INSET_M: f32 = 0.002;
const TICK_INSET_M: f32 = 0.004;

/// One wall's run, in rapier coordinates (z up): where its left-hand
/// corner is, the direction along it, and the inward normal.
struct Wall {
    start: Vec3,
    along: Vec3,
    inward: Vec3,
    /// The driver looks through this wall: ruler only, no squares.
    near: bool,
}

fn walls() -> [Wall; 4] {
    let h = ARENA_SIDE_M / 2.0;
    [
        // Far wall (+x), seen from inside: left is +y.
        Wall {
            start: Vec3::new(h, h, 0.0),
            along: Vec3::NEG_Y,
            inward: Vec3::NEG_X,
            near: false,
        },
        // Left wall (+y): runs toward -x.
        Wall {
            start: Vec3::new(-h, h, 0.0),
            along: Vec3::X,
            inward: Vec3::NEG_Y,
            near: false,
        },
        // Right wall (-y).
        Wall {
            start: Vec3::new(h, -h, 0.0),
            along: Vec3::NEG_X,
            inward: Vec3::Y,
            near: false,
        },
        // Near wall (-x), the driver's side.
        Wall {
            start: Vec3::new(-h, -h, 0.0),
            along: Vec3::Y,
            inward: Vec3::X,
            near: true,
        },
    ]
}

/// A thin panel `len` along the wall and `height` tall, its bottom-left
/// at `u` metres from the wall's start, `inset` in front of the wall.
fn panel(wall: &Wall, u: f32, len: f32, height: f32, inset: f32) -> Transform {
    let centre =
        wall.start + wall.along * (u + len / 2.0) + Vec3::Z * (height / 2.0) + wall.inward * inset;
    // Cuboid x along the wall, y through it. The third axis must make a
    // right-handed frame (along × inward is -z for every wall); the panel
    // is symmetric, so which way it points does not matter.
    let rot = Quat::from_mat3(&Mat3::from_cols(
        wall.along,
        wall.inward,
        wall.along.cross(wall.inward),
    ));
    Transform::from_translation(centre).with_rotation(rot)
}

pub fn spawn_wall_markings(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let paint = |c: Color| StandardMaterial {
        base_color: c,
        perceptual_roughness: 1.0,
        ..default()
    };
    let white = mats.add(paint(Color::srgb(0.92, 0.92, 0.9)));
    let black = mats.add(paint(Color::srgb(0.03, 0.03, 0.03)));
    let tick_mat = mats.add(paint(Color::srgb(0.95, 0.25, 0.1)));
    let unit = meshes.add(Cuboid::new(1.0, 0.002, 1.0));
    let add = |commands: &mut Commands,
               mat: &Handle<StandardMaterial>,
               t: Transform,
               len: f32,
               height: f32| {
        let e = commands
            .spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(mat.clone()),
                t.with_scale(Vec3::new(len, 1.0, height)),
            ))
            .id();
        commands.entity(root).add_child(e);
    };
    for wall in walls() {
        if !wall.near {
            let mut u = 0.0;
            let mut k = 0usize;
            while u < ARENA_SIDE_M - 1e-4 {
                let len = SQUARE_M.min(ARENA_SIDE_M - u);
                let mat = if k.is_multiple_of(2) { &white } else { &black };
                add(
                    commands,
                    mat,
                    panel(&wall, u, len, SQUARE_M, SQUARE_INSET_M),
                    len,
                    SQUARE_M,
                );
                u += SQUARE_M;
                k += 1;
            }
        }
        let ticks = (ARENA_SIDE_M / TICK_PITCH_M).floor() as usize;
        for i in 0..=ticks {
            let u = i as f32 * TICK_PITCH_M;
            let size = if i.is_multiple_of(10) {
                2
            } else if i.is_multiple_of(5) {
                1
            } else {
                0
            };
            let height = TICK_H[size];
            add(
                commands,
                &tick_mat,
                panel(&wall, u - TICK_W_M / 2.0, TICK_W_M, height, TICK_INSET_M),
                TICK_W_M,
                height,
            );
        }
    }
}
