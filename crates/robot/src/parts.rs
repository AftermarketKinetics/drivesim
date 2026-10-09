//! `parts.json` as the importer writes it (tools/import/step_import.py,
//! format 1): geometry per solid, SI units, the STEP's own frame.

use crate::math::{M3, V3};
use serde::Deserialize;

pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
pub struct PartsDoc {
    pub format: u32,
    pub source: String,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Part {
    pub id: usize,
    pub instance: String,
    pub product: String,
    pub path: String,
    pub color: Option<[f32; 3]>,
    pub mesh: String,
    pub bbox_m: [Option<V3>; 2],
    pub volume_m3: f64,
    pub com_m: V3,
    /// About `com_m`, at 1 kg/m³.
    pub inertia_m5: M3,
    pub rotation_axis: Option<Axis>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Axis {
    pub point_m: V3,
    pub dir: V3,
    pub curved_fraction: f64,
    pub cap_fraction: f64,
    pub radius_m: f64,
    pub width_m: f64,
    pub centre_m: V3,
    pub com_offset_m: f64,
}

#[derive(Debug)]
pub enum PartsError {
    Json(serde_json::Error),
    Format(u32),
}

impl std::fmt::Display for PartsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartsError::Json(e) => write!(f, "parts.json: {e}"),
            PartsError::Format(v) => write!(f, "parts.json format {v}, expected {FORMAT}"),
        }
    }
}

impl Part {
    /// True when `pattern` matches the instance name, the product name or
    /// the tree path: users name parts by whichever their CAD shows.
    pub fn matches(&self, pattern: &str) -> bool {
        use crate::glob::glob_match;
        glob_match(pattern, &self.instance)
            || glob_match(pattern, &self.product)
            || glob_match(pattern, &self.path)
    }
}

pub fn parse_parts(bytes: &[u8]) -> Result<PartsDoc, PartsError> {
    let doc: PartsDoc = serde_json::from_slice(bytes).map_err(PartsError::Json)?;
    if doc.format != FORMAT {
        return Err(PartsError::Format(doc.format));
    }
    Ok(doc)
}
