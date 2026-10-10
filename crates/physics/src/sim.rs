//! The fixed-step loop: physics at `PHYSICS_HZ`, control at `CONTROL_HZ`
//! (docs/drivesim.md §6: sized from the fastest rotor; a 10,000 rpm
//! weapon is unstable on a rapier joint at 1 ms and stable from 2 kHz).

use crate::arena::add_arena;
use crate::drivetrain::{Drivetrain, rotor_speed};
use crate::mixer::{Duties, MixerConfig, mix};
use crate::opponent::{COUNTDOWN_S, Opponent};
use crate::spawn::{RobotHandles, SpawnError, spawn_robot};
use crate::target::add_target;
use drivesim_input::Frame;
use drivesim_robot::robot_file::Control;
use drivesim_robot::{RobotModel, Rotor};
use rapier3d::prelude::*;

pub const PHYSICS_HZ: u32 = 4000;
pub const CONTROL_HZ: u32 = 1000;
pub const STEPS_PER_TICK: u32 = PHYSICS_HZ / CONTROL_HZ;
pub const GRAVITY: f32 = 9.81;

/// What the robot faces: an undriven copy of itself (the default), a
/// free box, or a copy the AI drives (user, 2026-10-10: a toggle on the
/// sim page). `Sim::versus` swaps the copy for another robot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpponentKind {
    #[default]
    Copy,
    Box,
    Ai,
}

/// Another robot to face in place of the copy (user, 2026-10-10: a public
/// robot picked for the AI on the sim page): its model and its meshes.
pub struct Rival<'a> {
    pub model: RobotModel,
    pub mesh: &'a dyn Fn(&str) -> Option<Vec<u8>>,
}

pub struct Sim {
    pub world: PhysicsWorld,
    pub robot: RobotHandles,
    pub model: RobotModel,
    pub opponent: Option<Opponent>,
    /// The box opponent's body, when `OpponentKind::Box` was chosen.
    pub target: Option<RigidBodyHandle>,
    mixer: Option<MixerConfig>,
    pub duties: Duties,
    drive: Drivetrain,
    /// Simulated seconds since the round started.
    pub time_s: f32,
}

impl Sim {
    /// The arena, the robot at (−1, 0) facing +x, and at (1, 0) either its
    /// opponent copy facing it (the AI's, for `Ai`) or a box as heavy as it.
    pub fn new(
        model: RobotModel,
        mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
        kind: OpponentKind,
    ) -> Result<Sim, SpawnError> {
        Self::versus(model, mesh, kind, None)
    }

    /// `new`, with `rival` (when given) in place of the copy for `Copy`
    /// and `Ai`. The box ignores it.
    pub fn versus(
        model: RobotModel,
        mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
        kind: OpponentKind,
        rival: Option<Rival>,
    ) -> Result<Sim, SpawnError> {
        let mut world = Self::empty_world(-GRAVITY);
        let _floor = add_arena(&mut world);
        let robot = spawn_robot(&mut world, &model, mesh, -1.0, 0.0, 0.0)?;
        Ok(match kind {
            OpponentKind::Copy | OpponentKind::Ai => {
                let (o_model, o_mesh) = match rival {
                    Some(r) => (r.model, r.mesh),
                    None => (model.clone(), mesh),
                };
                let handles =
                    spawn_robot(&mut world, &o_model, o_mesh, 1.0, 0.0, std::f32::consts::PI)?;
                let opponent = Opponent::new(handles, o_model, kind == OpponentKind::Ai);
                Self::assemble(world, robot, model, Some(opponent))
            }
            OpponentKind::Box => {
                let target = add_target(&mut world, 1.0, 0.0, model.total_mass() as f32);
                let mut s = Self::assemble(world, robot, model, None);
                s.target = Some(target);
                s
            }
        })
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
        // Speculative contacts within 5 mm, not the default 20 mm: at 4 kHz
        // a body at 20 m/s moves 5 mm a step, and the weapon has real CCD.
        // 20 mm gave a low robot (Obliterator 6 v72) 83 solver contacts at
        // rest. 2 mm saved little more and flipped the hit test's outcome.
        p.normalized_prediction_distance = 0.005;
        w
    }

    pub fn assemble(
        world: PhysicsWorld,
        robot: RobotHandles,
        model: RobotModel,
        opponent: Option<Opponent>,
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
            opponent,
            target: None,
            mixer,
            duties: Duties::default(),
            drive: Drivetrain::default(),
            time_s: 0.0,
        }
    }

    /// One control tick: decide duties from the radio frame (`None` = no
    /// radio) and the opponent's, then run `STEPS_PER_TICK` physics steps
    /// with them held.
    pub fn control_tick(&mut self, frame: Option<&Frame>) {
        let dt = 1.0 / CONTROL_HZ as f32;
        self.duties = match &self.mixer {
            Some(cfg) => mix(frame, cfg),
            None => Duties::default(),
        };
        if let Some(o) = &mut self.opponent {
            o.control(&self.world, self.robot.bodies[0], self.time_s, dt);
        }
        for _ in 0..STEPS_PER_TICK {
            self.physics_step();
        }
        self.time_s += dt;
    }

    /// Seconds left before the AI opponent starts; `None` when there is
    /// no AI or it has started.
    pub fn countdown_left(&self) -> Option<f32> {
        let ai = self.opponent.as_ref().is_some_and(|o| o.ai.is_some());
        (ai && self.time_s < COUNTDOWN_S).then_some(COUNTDOWN_S - self.time_s)
    }

    /// One physics step with the current duties: each robot's motor
    /// torques from its rotors' present speeds (`drivetrain.rs`).
    pub fn physics_step(&mut self) {
        self.drive
            .apply(&mut self.world, &self.model, &self.robot, &self.duties);
        if let Some(o) = &mut self.opponent {
            o.drive
                .apply(&mut self.world, &o.model, &o.robot, &o.duties);
        }
        self.world.step();
        self.drive.observe(&self.world, &self.model, &self.robot);
        if let Some(o) = &mut self.opponent {
            o.drive.observe(&self.world, &o.model, &o.robot);
        }
    }

    /// Rotor speed relative to the chassis about its axis, rad/s.
    pub fn rotor_speed(&self, rotor: &Rotor) -> f32 {
        rotor_speed(&self.world, &self.robot, rotor)
    }

    pub fn chassis(&self) -> &RigidBody {
        &self.world.bodies[self.robot.bodies[0]]
    }

    /// The opponent's chassis (the copy's, or the box), if there is one.
    pub fn opponent_chassis(&self) -> Option<&RigidBody> {
        self.opponent
            .as_ref()
            .map(|o| o.robot.bodies[0])
            .or(self.target)
            .map(|h| &self.world.bodies[h])
    }
}
