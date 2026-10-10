//! Fetches a robot bundle over HTTP and loads it with the same loader the
//! server's checker uses (docs/drivesim.md §5).
//!
//! The page URL says where the bundle is: `?robot=<id>` (served by
//! drivesim-server under /api) or `?bundle=<base url>/&motors=<url>` (a
//! static bundle, e.g. the fixture from `make web-dev`).

use drivesim_robot::parts::parse_parts;
use drivesim_robot::{RobotModel, load};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Everything the app needs, fetched: the model and every mesh by path.
pub struct Bundle {
    pub model: RobotModel,
    pub meshes: HashMap<String, Vec<u8>>,
    /// Part id -> its CAD colour (if the STEP had one), for drawing.
    pub part_colors: HashMap<usize, Option<[f32; 3]>>,
}

/// (bundle base URL ending in '/', motor library URL) from the page URL.
pub fn locate() -> Result<(String, String), String> {
    let search = web_sys::window()
        .ok_or("no window")?
        .location()
        .search()
        .map_err(|_| "no URL")?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).map_err(|_| "bad query")?;
    if let Some(id) = params.get("robot") {
        if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("bad robot id {id:?}"));
        }
        return Ok((
            format!("/api/robots/{id}/bundle/"),
            "/api/motors.toml".into(),
        ));
    }
    match (params.get("bundle"), params.get("motors")) {
        (Some(b), Some(m)) => Ok((if b.ends_with('/') { b } else { b + "/" }, m)),
        _ => Err("open this page with ?robot=<id> or ?bundle=<url>&motors=<url>".into()),
    }
}

async fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let window = web_sys::window().ok_or("no window")?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("{url}: {e:?}"))?
        .dyn_into()
        .map_err(|_| format!("{url}: not a response"))?;
    if !resp.ok() {
        return Err(format!("{url}: HTTP {}", resp.status()));
    }
    let buf = JsFuture::from(resp.array_buffer().map_err(|e| format!("{url}: {e:?}"))?)
        .await
        .map_err(|e| format!("{url}: {e:?}"))?;
    Ok(js_sys::Uint8Array::new(&buf).to_vec())
}

pub async fn fetch_bundle(base: String, motors_url: String) -> Result<Bundle, String> {
    let parts = fetch(&format!("{base}parts.json")).await?;
    let robot = fetch(&format!("{base}robot.toml")).await?;
    let motors = fetch(&motors_url).await?;
    let text = |b: Vec<u8>| String::from_utf8(b).map_err(|_| "not UTF-8".to_string());
    let model = load(&parts, &text(robot)?, &text(motors)?).map_err(|e| e.to_string())?;
    let doc = parse_parts(&parts).map_err(|e| e.to_string())?;
    let mut meshes = HashMap::new();
    for p in &doc.parts {
        meshes.insert(p.mesh.clone(), fetch(&format!("{base}{}", p.mesh)).await?);
    }
    let part_colors = doc.parts.iter().map(|p| (p.id, p.color)).collect();
    Ok(Bundle {
        model,
        meshes,
        part_colors,
    })
}
