//! The few vector and 3x3 operations the mass rollup needs, on plain
//! arrays so this crate carries no math dependency (f64: masses of parts
//! differing by 1e4 are summed).

pub type V3 = [f64; 3];
pub type M3 = [[f64; 3]; 3];

pub const ZERO: V3 = [0.0; 3];

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn mat_vec(m: &M3, v: V3) -> V3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}

pub fn mat_add(a: &M3, b: &M3) -> M3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][j] + b[i][j];
        }
    }
    r
}

pub fn mat_scale(a: &M3, k: f64) -> M3 {
    a.map(|row| row.map(|x| x * k))
}

/// `r · m · rᵀ`: a tensor expressed in a rotated frame.
pub fn rotate_tensor(r: &M3, m: &M3) -> M3 {
    let mut out = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut s = 0.0;
            for k in 0..3 {
                for l in 0..3 {
                    s += r[i][k] * m[k][l] * r[j][l];
                }
            }
            out[i][j] = s;
        }
    }
    out
}

/// Parallel-axis term for a point mass `m` at offset `d`:
/// `m (|d|² E − d dᵀ)`.
pub fn parallel_axis(m: f64, d: V3) -> M3 {
    let dd = dot(d, d);
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let e = if i == j { dd } else { 0.0 };
            r[i][j] = m * (e - d[i] * d[j]);
        }
    }
    r
}
