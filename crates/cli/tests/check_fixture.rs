//! drivesim-check on the fixture bundle, run as the server runs it.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_drivesim-check");
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn run(bundle: &str) -> (i32, serde_json::Value) {
    let out = Command::new(BIN)
        .args([bundle, &format!("{ROOT}/data/motors.toml")])
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        serde_json::from_slice(&out.stdout).unwrap(),
    )
}

#[test]
fn the_fixture_reports_its_numbers() {
    let (code, r) = run(&format!("{ROOT}/fixtures/simple_bot"));
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["name"], "simple_bot");
    assert!(
        (r["total_mass_kg"].as_f64().unwrap() - 5.265).abs() < 0.01,
        "{}",
        r["total_mass_kg"]
    );
    assert_eq!(r["wheels"].as_array().unwrap().len(), 2);
    assert_eq!(r["weapons"][0]["name"], "disc");
    assert_eq!(r["rest"]["settled"], true);
    let touching = r["rest"]["touching_floor"].as_array().unwrap();
    assert!(!touching.contains(&serde_json::json!("disc")));
}

#[test]
fn a_bad_bundle_is_a_json_error_not_a_crash() {
    let dir = std::env::temp_dir().join("drivesim-check-bad");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(
        format!("{ROOT}/fixtures/simple_bot/parts.json"),
        dir.join("parts.json"),
    )
    .unwrap();
    std::fs::write(dir.join("robot.toml"), "[robot]\nname = 1\n").unwrap();
    let (code, r) = run(dir.to_str().unwrap());
    assert_eq!(code, 1);
    assert_eq!(r["ok"], false);
    assert!(r["error"].as_str().unwrap().contains("robot.toml"));
}
