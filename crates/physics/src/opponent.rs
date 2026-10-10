//! The opponent robot (the driven robot's copy, or a rival picked on the
//! sim page): either undriven (braked, weapon off) or driven by
//! `ai_driver` once the countdown ends (user, 2026-10-10).

use crate::ai_driver::{AiDriver, Pose};
use crate::drivetrain::Drivetrain;
use crate::mixer::Duties;
use crate::spawn::RobotHandles;
use drivesim_robot::RobotModel;
use rapier3d::prelude::*;

/// What an undriven opponent's ESCs get: armed with its sticks centred,
/// so its wheels brake and its weapon is off. It holds its ground like an
/// idle robot rather than rolling away on free wheels. The AI holds this
/// too until the countdown ends.
pub const OPPONENT_DUTIES: Duties = Duties {
    left: Some(0.0),
    right: Some(0.0),
    weapon: Some(0.0),
};

/// Seconds of sim time before the AI opponent starts (user, 2026-10-10:
/// a 3-2-1 countdown, then it fights).
pub const COUNTDOWN_S: f32 = 3.0;

/// The opponent robot: its bodies and its own model (a clone of the
/// driven robot's for the copy).
pub struct Opponent {
    pub robot: RobotHandles,
    pub model: RobotModel,
    pub(crate) drive: Drivetrain,
    /// `Some` when the AI drives it.
    pub ai: Option<AiDriver>,
    /// The duties held over the current control tick.
    pub duties: Duties,
}

/// A chassis as the AI sees it: on the floor, heading the way its wheels
/// drive it. Upside down that is its −x axis: the floor meets each wheel
/// on the other side of the hub, so forward spin rolls it backwards,
/// while its left wheels stay on the left of that way.
pub fn pose_of(body: &RigidBody) -> Pose {
    let p = body.translation();
    let up = (*body.rotation() * Vector::Z).z;
    let fwd = *body.rotation() * Vector::X * if up < 0.0 { -1.0 } else { 1.0 };
    Pose {
        x: p.x,
        y: p.y,
        heading: fwd.y.atan2(fwd.x),
        speed: body.linvel().dot(fwd),
        yaw_rate: body.angvel().z,
    }
}

impl Opponent {
    pub fn new(robot: RobotHandles, model: RobotModel, ai: bool) -> Self {
        Opponent {
            robot,
            model,
            drive: Drivetrain::default(),
            ai: ai.then(AiDriver::default),
            duties: OPPONENT_DUTIES,
        }
    }

    /// Decides this control tick's duties: the AI's once `time_s` is past
    /// the countdown, `OPPONENT_DUTIES` otherwise.
    pub fn control(&mut self, world: &PhysicsWorld, target: RigidBodyHandle, time_s: f32, dt: f32) {
        self.duties = match &mut self.ai {
            Some(ai) if time_s >= COUNTDOWN_S => {
                let me = pose_of(&world.bodies[self.robot.bodies[0]]);
                ai.tick(&me, &pose_of(&world.bodies[target]), dt)
            }
            _ => OPPONENT_DUTIES,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn body(rot: Rotation) -> RigidBody {
        RigidBodyBuilder::dynamic()
            .rotation(rot.to_scaled_axis())
            .linvel(Vector::new(1.0, 0.0, 0.0))
            .build()
    }

    #[test]
    fn upright_heads_along_plus_x() {
        let p = pose_of(&body(Rotation::from_rotation_z(0.5)));
        assert!((p.heading - 0.5).abs() < 1e-5, "{p:?}");
    }

    #[test]
    fn rolled_over_heads_along_minus_x() {
        let p = pose_of(&body(Rotation::from_rotation_x(PI)));
        assert!((p.heading.abs() - PI).abs() < 1e-5, "{p:?}");
        assert!((p.speed + 1.0).abs() < 1e-5, "{p:?}");
    }
}
