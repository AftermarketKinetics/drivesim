//! Which of a weapon's parts get collision hulls (user, 2026-10-10: "do
//! the automatic one"): only those reaching near its outer radius. Parts
//! deep inside (spindle, pulley, axle, spacers) can't touch anything, yet
//! each hull is swept at weapon speed every step, the bulk of a fight's
//! physics cost (lesson `spinning-weapon-hull-count-dominates-step-cost`).

use drivesim_robot::stl::Tri;

/// A part collides if it reaches at least this fraction of the weapon's
/// outermost radius from its axis.
pub const REACH_FRACTION: f32 = 0.5;

/// The furthest any vertex of `part` lies from the axis through `point`
/// along `dir` (unit), m.
pub fn part_reach(part: &[Tri], point: [f32; 3], dir: [f32; 3]) -> f32 {
    part.iter()
        .flatten()
        .map(|p| {
            let r = [p[0] - point[0], p[1] - point[1], p[2] - point[2]];
            let along = r[0] * dir[0] + r[1] * dir[1] + r[2] * dir[2];
            let perp = [
                r[0] - along * dir[0],
                r[1] - along * dir[1],
                r[2] - along * dir[2],
            ];
            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt()
        })
        .fold(0.0, f32::max)
}

/// The parts of a weapon body that reach `REACH_FRACTION` of the farthest
/// one's radius, in their original order.
pub fn reaching_parts(parts: &[Vec<Tri>], point: [f32; 3], dir: [f32; 3]) -> Vec<Vec<Tri>> {
    let reach: Vec<f32> = parts.iter().map(|p| part_reach(p, point, dir)).collect();
    let outer = reach.iter().copied().fold(0.0, f32::max);
    parts
        .iter()
        .zip(&reach)
        .filter(|(_, r)| **r >= REACH_FRACTION * outer)
        .map(|(p, _)| p.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One triangle with a vertex `r` m out along +x, about the z axis.
    fn part(r: f32) -> Vec<Tri> {
        vec![[[0.0, 0.0, 0.0], [r, 0.0, 0.0], [0.0, 0.0, 0.01]]]
    }

    const O: [f32; 3] = [0.0, 0.0, 0.0];
    const Z: [f32; 3] = [0.0, 0.0, 1.0];

    #[test]
    fn reach_ignores_distance_along_the_axis() {
        let t = vec![[[0.0, 0.0, 5.0], [0.0, 0.03, 0.0], [0.0, 0.0, -5.0]]];
        assert!((part_reach(&t, O, Z) - 0.03).abs() < 1e-6);
    }

    #[test]
    fn inner_parts_are_dropped_and_outer_ones_kept() {
        let kept = reaching_parts(&[part(0.2), part(0.03), part(0.15), part(0.09)], O, Z);
        let r: Vec<f32> = kept.iter().map(|p| part_reach(p, O, Z)).collect();
        assert_eq!(r, vec![0.2, 0.15]);
    }

    #[test]
    fn a_single_part_is_always_kept() {
        assert_eq!(reaching_parts(&[part(0.01)], O, Z).len(), 1);
    }
}
