//! Collision shapes from a part's mesh (robot-frame triangles).

use drivesim_robot::stl::Tri;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// A hull more than this many times the part's own volume is too poor a
/// fit (an L-bracket, a ring), and the part is decomposed instead.
pub const HULL_SLACK: f32 = 1.5;

/// Convex hull of the part, or a V-HACD decomposition when the hull
/// overfills it by more than `HULL_SLACK`. `None` for a degenerate mesh.
pub fn part_shape(tris: &[Tri]) -> Option<SharedShape> {
    let (verts, idx) = indexed(tris);
    let hull = SharedShape::convex_hull(&verts)?;
    let volume = mesh_volume(tris);
    let hull_volume = hull.mass_properties(1.0).mass();
    if volume > 0.0 && hull_volume > HULL_SLACK * volume {
        Some(SharedShape::convex_decomposition(&verts, &idx))
    } else {
        Some(hull)
    }
}

/// Enclosed volume of a closed, outward-wound mesh (divergence theorem).
pub fn mesh_volume(tris: &[Tri]) -> f32 {
    tris.iter()
        .map(|[a, b, c]| Vector::from(*a).dot(Vector::from(*b).cross(Vector::from(*c))) / 6.0)
        .sum()
}

/// STL triangles share no vertices; V-HACD wants an indexed mesh. Vertices
/// within a micrometre are merged.
fn indexed(tris: &[Tri]) -> (Vec<Vector>, Vec<[u32; 3]>) {
    let mut map: HashMap<[i64; 3], u32> = HashMap::new();
    let mut verts = Vec::new();
    let idx = tris
        .iter()
        .map(|t| {
            t.map(|p| {
                let key = p.map(|x| (x as f64 * 1e6).round() as i64);
                *map.entry(key).or_insert_with(|| {
                    verts.push(Vector::from(p));
                    (verts.len() - 1) as u32
                })
            })
        })
        .collect();
    (verts, idx)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let s = part_shape(&t).unwrap();
        assert!(s.as_convex_polyhedron().is_some());
    }

    #[test]
    fn an_l_shape_is_decomposed() {
        let mut t = box_tris([0.0; 3], [0.2, 0.02, 0.02]);
        t.extend(box_tris([0.0, 0.0, 0.02], [0.02, 0.02, 0.2]));
        // Two boxes touching on a face: not one closed mesh, but the volume
        // still sums, and the hull (a big triangle prism) overfills it.
        let s = part_shape(&t).unwrap();
        assert!(s.as_compound().is_some());
    }
}
