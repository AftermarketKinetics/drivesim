//! drivesim-check BUNDLE_DIR MOTORS_TOML
//!
//! Loads BUNDLE_DIR/{parts.json, robot.toml, meshes/} exactly as the sim
//! would, settles it on the floor, and prints one JSON object (see
//! report.rs). Exit 0 when the robot loads, 1 when it does not, 2 on a
//! usage or I/O error.

mod report;

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let [_, bundle, motors] = args.as_slice() else {
        eprintln!("usage: drivesim-check BUNDLE_DIR MOTORS_TOML");
        return ExitCode::from(2);
    };
    let dir = Path::new(bundle);
    let read = |name: &str| std::fs::read(dir.join(name));
    let (parts, robot, motors) = match (
        read("parts.json"),
        read("robot.toml"),
        std::fs::read(motors),
    ) {
        (Ok(p), Ok(r), Ok(m)) => (p, r, m),
        (p, r, m) => {
            let missing: Vec<&str> = [
                ("parts.json", p.is_err()),
                ("robot.toml", r.is_err()),
                ("motors", m.is_err()),
            ]
            .iter()
            .filter(|(_, bad)| *bad)
            .map(|(n, _)| *n)
            .collect();
            let out = serde_json::json!({"ok": false, "error": format!("cannot read {}", missing.join(", "))});
            println!("{out}");
            return ExitCode::from(1);
        }
    };
    let text = |b: Vec<u8>| String::from_utf8_lossy(&b).into_owned();
    let out = report::report(&parts, &text(robot), &text(motors), &|p| {
        std::fs::read(dir.join(p)).ok()
    });
    println!("{out}");
    if out["ok"] == true {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
