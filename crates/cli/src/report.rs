//! A robot bundle -> the JSON report the role conversation reads
//! (docs/drivesim.md §4): load errors, or masses, inertia, rotors and how
//! the robot rests on the floor.

use drivesim_physics::motor::no_load_speed;
use drivesim_physics::rest::settle;
use drivesim_robot::{RobotModel, load};
use serde_json::{Value, json};

/// Seconds the robot is left to settle on the floor.
pub const SETTLE_S: f32 = 2.0;

fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// `{"ok": false, "error": "..."}` for a bundle that does not load or
/// spawn; otherwise `{"ok": true, ...}` with the robot's numbers.
pub fn report(
    parts_json: &[u8],
    robot_toml: &str,
    motors_toml: &str,
    mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Value {
    let model = match load(parts_json, robot_toml, motors_toml) {
        Ok(m) => m,
        Err(e) => return json!({"ok": false, "error": e.to_string()}),
    };
    let rest = match settle(&model, mesh, SETTLE_S) {
        Ok(r) => r,
        Err(e) => return json!({"ok": false, "error": e.to_string()}),
    };
    let all = model.combined();
    json!({
        "ok": true,
        "name": model.name,
        "total_mass_kg": r3(all.mass),
        "com_m": all.com.map(r3),
        "inertia_kg_m2": all.inertia.map(|row| row.map(|x| (x * 1e6).round() / 1e6)),
        "bodies": model.bodies.iter().map(|b| json!({
            "name": b.name,
            "parts": b.parts.len(),
            "mass_kg": r3(b.mass.mass),
        })).collect::<Vec<_>>(),
        "wheels": rotors(&model, &model.wheels),
        "weapons": rotors(&model, &model.weapons),
        "rest": {
            "tilt_deg": (rest.tilt_deg as f64 * 10.0).round() / 10.0,
            "com_height_mm": (rest.com_height_m as f64 * 1e4).round() / 10.0,
            "touching_floor": rest.touching,
            "settled": rest.residual_speed < 0.01,
        },
    })
}

fn rotors(model: &RobotModel, list: &[drivesim_robot::Rotor]) -> Vec<Value> {
    let v = model.battery_v as f32;
    list.iter()
        .map(|r| {
            let w = no_load_speed(1.0, r, v) as f64;
            json!({
                "name": r.name,
                "axis_dir": r.axis_dir.map(r3),
                "axis_point_m": r.axis_point.map(r3),
                "radius_mm": (r.radius_m * 1e4).round() / 10.0,
                "no_load_rpm": (w * 60.0 / std::f64::consts::TAU).round(),
                "no_load_tip_speed_m_s": r3(w * r.radius_m),
                "side": r.side.map(|s| format!("{s:?}").to_lowercase()),
                "mu": r.mu,
                "motor_kv": r.motor.kv_rpm_per_v,
                "provisional_motor_fields": r.motor.provisional,
            })
        })
        .collect()
}
