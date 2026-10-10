//! Collision shapes for a body, from its parts' meshes (robot-frame
//! triangles).

use drivesim_robot::stl::Tri;
use rapier3d::prelude::*;

/// A body collides as the convex hulls of at most this many of its parts,
/// the biggest by volume. The first real robot (Seraphs V1, 2026-10-10)
/// had 349 parts: a shape per part, with V-HACD for the concave ones, made
/// 695 colliders (3,619 convex pieces) across it and its opponent copy,
/// 46 s to build and 0.01x real time. Twelve hulls per body ran it at 6.3x.
/// Small parts (fasteners, magnets, internals) get no shape; their mass
/// still counts, from the model.
pub const MAX_HULLS_PER_BODY: usize = 12;

/// A hull keeps at most this many vertices. Collision and the per-step
/// bounding box cost grow with them: Obliterator 6 v72 (2026-10-10) had
/// hulls of up to 260 vertices, 2,336 in all; 32 each took its step from
/// 70 to 54 µs.
pub const MAX_HULL_VERTS: usize = 32;

/// The convex hulls of the body's `MAX_HULLS_PER_BODY` biggest parts,
/// each cut to `MAX_HULL_VERTS`. Degenerate meshes (no hull) are skipped.
pub fn body_shapes(parts: &[Vec<Tri>]) -> Vec<SharedShape> {
    let mut by_size: Vec<(f32, &Vec<Tri>)> =
        parts.iter().map(|t| (mesh_volume(t).abs(), t)).collect();
    by_size.sort_by(|a, b| b.0.total_cmp(&a.0));
    by_size
        .into_iter()
        .take(MAX_HULLS_PER_BODY)
        .filter_map(|(_, t)| {
            let pts: Vec<Vector> = t.iter().flatten().map(|p| Vector::from(*p)).collect();
            let hull = SharedShape::convex_hull(&pts)?;
            let verts = hull.as_convex_polyhedron()?.points();
            if verts.len() <= MAX_HULL_VERTS {
                return Some(hull);
            }
            SharedShape::convex_hull(&spread_subset(verts, MAX_HULL_VERTS))
        })
        .collect()
}

/// `n` of `pts`, spread out: the six axis extremes first (so the bounding
/// box, and the lowest point the robot rests on, are kept exactly), then
/// each next point the one farthest from those already chosen.
fn spread_subset(pts: &[Vector], n: usize) -> Vec<Vector> {
    let extreme = |axis: usize, max: bool| {
        (0..pts.len())
            .max_by(|&a, &b| {
                let o = pts[a][axis].total_cmp(&pts[b][axis]);
                if max { o } else { o.reverse() }
            })
            .unwrap_or(0)
    };
    let mut keep: Vec<usize> = Vec::new();
    for i in (0..3).flat_map(|a| [extreme(a, false), extreme(a, true)]) {
        if !keep.contains(&i) {
            keep.push(i);
        }
    }
    let dist = |j: usize, keep: &[usize]| {
        keep.iter()
            .map(|&k| (pts[j] - pts[k]).length_squared())
            .fold(f32::INFINITY, f32::min)
    };
    let mut d: Vec<f32> = (0..pts.len()).map(|j| dist(j, &keep)).collect();
    while keep.len() < n.min(pts.len()) {
        let i = (0..pts.len())
            .max_by(|&a, &b| d[a].total_cmp(&d[b]))
            .unwrap();
        keep.push(i);
        for (j, dj) in d.iter_mut().enumerate() {
            *dj = dj.min((pts[j] - pts[i]).length_squared());
        }
    }
    keep.iter().map(|&i| pts[i]).collect()
}

/// Enclosed volume of a closed, outward-wound mesh (divergence theorem).
pub fn mesh_volume(tris: &[Tri]) -> f32 {
    tris.iter()
        .map(|[a, b, c]| Vector::from(*a).dot(Vector::from(*b).cross(Vector::from(*c))) / 6.0)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{PI, TAU};

    /// A closed, outward-wound axis-aligned box from `lo` to `hi`.
    pub fn box_tris(lo: [f32; 3], hi: [f32; 3]) -> Vec<Tri> {
        let c = |i: usize| {
            [
                if i & 1 == 0 { lo[0] } else { hi[0] },
                if i & 2 == 0 { lo[1] } else { hi[1] },
                if i & 4 == 0 { lo[2] } else { hi[2] },
            ]
        };
        // Faces as corner indices, counter-clockwise seen from outside.
        let quads = [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ];
        quads
            .iter()
            .flat_map(|q| [[c(q[0]), c(q[1]), c(q[2])], [c(q[0]), c(q[2]), c(q[3])]])
            .collect()
    }

    #[test]
    fn a_box_is_its_own_hull() {
        let t = box_tris([0.0; 3], [0.2, 0.1, 0.05]);
        assert!((mesh_volume(&t) - 0.001).abs() < 1e-9);
        let s = body_shapes(&[t]);
        assert_eq!(s.len(), 1);
        assert!(s[0].as_convex_polyhedron().is_some());
    }

    #[test]
    fn a_round_part_is_cut_to_the_vertex_cap_keeping_its_bounds() {
        // A UV sphere of radius 0.05 m with 24 × 12 bands: ~260 hull vertices.
        let (nu, nv, r) = (24, 12, 0.05f32);
        let p = |i: usize, j: usize| {
            let (u, v) = (i as f32 / nu as f32 * TAU, j as f32 / nv as f32 * PI);
            [r * v.sin() * u.cos(), r * v.sin() * u.sin(), r * v.cos()]
        };
        let t: Vec<Tri> = (0..nu)
            .flat_map(|i| {
                (0..nv).flat_map(move |j| {
                    [
                        [p(i, j), p(i, j + 1), p(i + 1, j + 1)],
                        [p(i, j), p(i + 1, j + 1), p(i + 1, j)],
                    ]
                })
            })
            .collect();
        let s = body_shapes(&[t]);
        let hull = s[0].as_convex_polyhedron().unwrap();
        assert!(
            hull.points().len() <= MAX_HULL_VERTS,
            "{}",
            hull.points().len()
        );
        let b = s[0].compute_local_aabb();
        assert!((b.mins.z + r).abs() < 1e-6 && (b.maxs.z - r).abs() < 1e-6);
        assert!((b.mins.x + r).abs() < 1e-6 && (b.maxs.x - r).abs() < 1e-6);
    }

    #[test]
    fn only_the_biggest_parts_get_shapes() {
        // 20 cubes of growing size: the 12 biggest get hulls.
        let parts: Vec<Vec<Tri>> = (1..=20)
            .map(|i| {
                let a = i as f32 * 0.01;
                box_tris([i as f32, 0.0, 0.0], [i as f32 + a, a, a])
            })
            .collect();
        let s = body_shapes(&parts);
        assert_eq!(s.len(), MAX_HULLS_PER_BODY);
        let smallest = s
            .iter()
            .map(|sh| sh.mass_properties(1.0).mass())
            .fold(f32::INFINITY, f32::min);
        assert!(
            (smallest - 0.09f32.powi(3)).abs() < 1e-6,
            "smallest hull {smallest}"
        );
    }
}
