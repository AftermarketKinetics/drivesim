//! Where the robots take large impacts, for the glowing marks the sim
//! page draws (user, 2026-10-11: "glowing orange-to-red marks where they
//! take large impacts, scaling based on the strength of the impact").

use crate::impact_loss::Group;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// A contact counts as a large impact when, in one physics step, its
/// impulse would change its group's speed by at least this much, m/s.
pub const MARK_DV: f32 = 2.0;

/// One large impact on one body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impact {
    pub body: RigidBodyHandle,
    /// The contact point on the body's surface, in the body's frame.
    pub point: Vector,
    /// The step's contact impulse over the group's mass, m/s: the speed
    /// it would give the whole robot, so classes compare.
    pub dv: f32,
}

/// The large impacts of the step just taken: each contact pair between a
/// group and anything but `floor` and itself, at its strongest point, once
/// per group body it touches.
pub fn find_impacts(world: &PhysicsWorld, groups: &[Group], floor: ColliderHandle) -> Vec<Impact> {
    let mut group_of = HashMap::new();
    for (i, g) in groups.iter().enumerate() {
        for h in &g.bodies {
            group_of.insert(*h, i);
        }
    }
    let mass: Vec<f32> = groups
        .iter()
        .map(|g| g.bodies.iter().map(|h| world.bodies[*h].mass()).sum())
        .collect();
    let mut out = Vec::new();
    for pair in world.narrow_phase.contact_pairs() {
        let (c1, c2) = (pair.collider1, pair.collider2);
        if c1 == floor || c2 == floor {
            continue;
        }
        let (b1, b2) = (world.colliders[c1].parent(), world.colliders[c2].parent());
        let (g1, g2) = (
            b1.and_then(|b| group_of.get(&b).copied()),
            b2.and_then(|b| group_of.get(&b).copied()),
        );
        if g1 == g2 {
            continue;
        }
        let j = pair.total_impulse_magnitude();
        let Some(p) = pair
            .solver_manifolds()
            .iter()
            .flat_map(|m| &m.points)
            .max_by(|a, b| a.data.impulse.total_cmp(&b.data.impulse))
        else {
            continue;
        };
        for (c, b, g, local) in [(c1, b1, g1, p.local_p1), (c2, b2, g2, p.local_p2)] {
            let (Some(b), Some(g)) = (b, g) else {
                continue;
            };
            let dv = j / mass[g];
            if dv < MARK_DV {
                continue;
            }
            let world_p = world.colliders[c].position().transform_point(local);
            out.push(Impact {
                body: b,
                point: world.bodies[b].position().inverse_transform_point(world_p),
                dv,
            });
        }
    }
    out
}
