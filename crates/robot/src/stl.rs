//! Binary STL bytes -> triangles. The importer writes metres, wound
//! outward (tools/import/tessellate.py).

pub type Tri = [[f32; 3]; 3];

/// `None` if the bytes are not a complete binary STL.
pub fn parse_stl(bytes: &[u8]) -> Option<Vec<Tri>> {
    let n = u32::from_le_bytes(bytes.get(80..84)?.try_into().ok()?) as usize;
    if bytes.len() < 84 + n * 50 {
        return None;
    }
    let f = |o: usize| f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    Some(
        (0..n)
            .map(|i| {
                let base = 84 + i * 50 + 12; // skip the facet normal
                let v = |k: usize| {
                    let o = base + k * 12;
                    [f(o), f(o + 4), f(o + 8)]
                };
                [v(0), v(1), v(2)]
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_triangle() -> Vec<u8> {
        let mut b = vec![0u8; 80];
        b.extend(1u32.to_le_bytes());
        for x in [
            0.0f32, 0.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
        ] {
            b.extend(x.to_le_bytes());
        }
        b.extend([0, 0]);
        b
    }

    #[test]
    fn reads_vertices_and_skips_the_normal() {
        let t = parse_stl(&one_triangle()).unwrap();
        assert_eq!(t, vec![[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]]);
    }

    #[test]
    fn truncated_is_none() {
        let b = one_triangle();
        assert!(parse_stl(&b[..b.len() - 1]).is_none());
        assert!(parse_stl(&b[..40]).is_none());
    }
}
