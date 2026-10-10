//! The fixed-step loop: physics at `PHYSICS_HZ`, control at `CONTROL_HZ`
//! (docs/drivesim.md §6: sized from the fastest rotor; a 10,000 rpm
//! weapon is unstable on a rapier joint at 1 ms and stable from 2 kHz).

use crate::arena::add_arena;
use crate::drivetrain::{Drivetrain, rotor_speed};
use crate::mixer::{Duties, MixerConfig, mix};
use crate::spawn::{RobotHandles, SpawnError, spawn_robot};
use drivesim_input::Frame;
use drivesim_robot::robot_file::Control;
use drivesim_robot::{RobotModel, Rotor};
use rapier3d::prelude::*;

pub const PHYSICS_HZ: u32 = 4000;
pub const CONTROL_HZ: u32 = 1000;
pub const STEPS_PER_TICK: u32 = PHYSICS_HZ / CONTROL_HZ;
pub const GRAVITY: f32 = 9.81;

/// What the opponent's ESCs get: armed with its sticks centred, so its
/// wheels brake and its weapon is off. It holds its ground like an idle
/// robot rather than rolling away on free wheels.
pub const OPPONENT_DUTIES: Duties = Duties {
    left: Some(0.0),
    right: Some(0.0),
    weapon: Some(0.0),
};

/// An undriven copy of the robot (user, 2026-10-10: "replace the box with
/// a copy of simple bot"), sharing the driven robot's model.
pub struct Opponent {
    pub robot: RobotHandles,
    drive: Drivetrain,
}

pub struct Sim {
    pub world: PhysicsWorld,
    pub robot: RobotHandles,
    pub model: RobotModel,
    pub opponent: Option<Opponent>,
    mixer: Option<MixerConfig>,
    pub duties: Duties,
    drive: Drivetrain,
}

impl Sim {
    /// The arena, the robot at (−1, 0) facing +x, and its opponent copy at
    /// (1, 0) facing it.
    pub fn new(
        model: RobotModel,
        mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> Result<Sim, SpawnError> {
        let mut world = Self::empty_world(-GRAVITY);
        let _floor = add_arena(&mut world);
        let robot = spawn_robot(&mut world, &model, mesh, -1.0, 0.0, 0.0)?;
        let opponent = spawn_robot(&mut world, &model, mesh, 1.0, 0.0, std::f32::consts::PI)?;
        Ok(Self::assemble(world, robot, model, Some(opponent)))
    }

    /// A world with nothing in it, for tests that build their own scene.
    pub fn empty_world(gravity_z: f32) -> PhysicsWorld {
        let mut w = PhysicsWorld::new();
        w.gravity = Vector::new(0.0, 0.0, gravity_z);
        let p = &mut w.integration_parameters;
        p.dt = 1.0 / PHYSICS_HZ as f32;
        // Contact as exact as rapier offers (user, 2026-10-10, for ~1.7x
        // the physics cost): one Coulomb friction cone per contact point,
        // not rapier's default single cone plus a "twist" constraint per
        // patch; fresh contact points every step, not ones recycled while
        // a pair barely moved (stale on a spinning wheel); and floor
        // contact as stiff springs (200/400 Hz, critically damped) instead
        // of the default 30/60 Hz with ζ = 10, tuned for piles of boxes.
        p.friction_model = FrictionModel::Coulomb;
        p.contact_recycling = false;
        p.contact_softness = SpringCoefficients::new(200.0, 1.0);
        p.static_contact_softness = SpringCoefficients::new(400.0, 1.0);
        p.num_solver_iterations = 4;
        p.warmstart_joints = true;
        w
    }

    pub fn assemble(
        world: PhysicsWorld,
        robot: RobotHandles,
        model: RobotModel,
        opponent: Option<RobotHandles>,
    ) -> Sim {
        let mixer = match model.control {
            Control::Mixer {
                throttle,
                turn,
                weapon,
                arm,
                reverse_turn,
            } => Some(MixerConfig {
                throttle,
                turn,
                weapon,
                arm,
                reverse_turn,
            }),
            // The momentum-core hook is not built yet: the robot sits idle.
            Control::MomentumCore => None,
        };
        Sim {
            world,
            robot,
            model,
            opponent: opponent.map(|robot| Opponent {
                robot,
                drive: Drivetrain::default(),
            }),
            mixer,
            duties: Duties::default(),
            drive: Drivetrain::default(),
        }
    }

    /// One control tick: decide duties from the radio frame (`None` = no
    /// radio), then run `STEPS_PER_TICK` physics steps with them held.
    pub fn control_tick(&mut self, frame: Option<&Frame>) {
        self.duties = match &self.mixer {
            Some(cfg) => mix(frame, cfg),
            None => Duties::default(),
        };
        for _ in 0..STEPS_PER_TICK {
            self.physics_step();
        }
    }

    /// One physics step with the current duties: each robot's motor
    /// torques from its rotors' present speeds (`drivetrain.rs`).
    pub fn physics_step(&mut self) {
        self.drive
            .apply(&mut self.world, &self.model, &self.robot, &self.duties);
        if let Some(o) = &mut self.opponent {
            o.drive
                .apply(&mut self.world, &self.model, &o.robot, &OPPONENT_DUTIES);
        }
        self.world.step();
        self.drive.observe(&self.world, &self.model, &self.robot);
        if let Some(o) = &mut self.opponent {
            o.drive.observe(&self.world, &self.model, &o.robot);
        }
    }

    /// Rotor speed relative to the chassis about its axis, rad/s.
    pub fn rotor_speed(&self, rotor: &Rotor) -> f32 {
        rotor_speed(&self.world, &self.robot, rotor)
    }

    pub fn chassis(&self) -> &RigidBody {
        &self.world.bodies[self.robot.bodies[0]]
    }

    /// The opponent's chassis, if there is one.
    pub fn opponent_chassis(&self) -> Option<&RigidBody> {
        self.opponent
            .as_ref()
            .map(|o| &self.world.bodies[o.robot.bodies[0]])
    }
}
