//! Shared by the physics tests: the fixture robot (fixtures/simple_bot)
//! and radio frames built the way the web app builds them.

#![allow(dead_code)]

use drivesim_input::{Frame, frame_from_axes};
use drivesim_robot::{RobotModel, load};
use std::path::Path;

pub fn fixture_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/simple_bot"
    ))
}

pub fn fixture_model() -> RobotModel {
    let d = fixture_dir();
    load(
        &std::fs::read(d.join("parts.json")).unwrap(),
        &std::fs::read_to_string(d.join("robot.toml")).unwrap(),
        include_str!("../../../../data/motors.toml"),
    )
    .unwrap()
}

/// Tests read meshes from disk; the app fetches them over HTTP.
pub fn fixture_mesh(path: &str) -> Option<Vec<u8>> {
    std::fs::read(fixture_dir().join(path)).ok()
}

/// Sticks and switches as -1..+1, for the fixture's mixer: ch1 turn,
/// ch2 throttle, ch3 weapon (-1 = off), ch5 arm.
pub fn radio(turn: f32, throttle: f32, weapon: f32, armed: bool) -> Frame {
    let arm = if armed { 1.0 } else { -1.0 };
    frame_from_axes(Some(&[turn, throttle, weapon, 0.0, arm])).unwrap()
}
