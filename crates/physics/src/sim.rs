//! The fixed-step loop: physics at `PHYSICS_HZ` (or another of
//! `PHYSICS_RATES_HZ`), control at `CONTROL_HZ` (docs/drivesim.md §6:
//! sized from the fastest rotor; a 10,000 rpm weapon is unstable on a
//! rapier joint at 1 ms and stable from 2 kHz).

use crate::ai_driver::AiDriver;
use crate::arena::add_arena;
use crate::drivetrain::{Drivetrain, rotor_speed};
use crate::impact_loss::{Group, absorb, snapshot};
use crate::impacts::{Impact, find_impacts};
use crate::mixer::{Duties, MixerConfig, mix};
use crate::opponent::{BOUT_S, COUNTDOWN_S, OPPONENT_DUTIES, Opponent, pose_of, weapon_fraction};
use crate::spawn::{RobotHandles, SpawnError, spawn_robot, v};
use crate::target::add_target;
use drivesim_input::Frame;
use drivesim_robot::robot_file::Control;
use drivesim_robot::{RobotModel, Rotor};
use rapier3d::prelude::*;

/// The physics rate a `Sim` starts at.
pub const PHYSICS_HZ: u32 = 2000;
/// The rates `Sim::set_physics_hz` takes (user, 2026-10-11: a 2–4 kHz
/// slider on the sim page, 2 kHz by default): whole steps per control
/// tick. A light robot's small, fast weapon needs 4 kHz: a 0.96 kg beetle's
/// 45 mm bar at 10.6k rpm made energy from nothing in its hits at 2 and
/// 3 kHz, and stayed physical at 4.
pub const PHYSICS_RATES_HZ: [u32; 3] = [2000, 3000, 4000];
pub const CONTROL_HZ: u32 = 1000;
pub const GRAVITY: f32 = 9.81;

/// What the robot faces: an undriven copy of itself (the default), a
/// free box, or a copy the AI drives (user, 2026-10-10: a toggle on the
/// sim page); or, for `AiVsAi`, the AI's copy with a second AI driving the
/// robot too, so it can be tested without a radio (user, 2026-10-11).
/// `Sim::versus` swaps the copy for another robot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpponentKind {
    #[default]
    Copy,
    Box,
    Ai,
    AiVsAi,
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
    /// The arena floor, exempt from impact loss (`impact_loss.rs`); set
    /// by `new`. `None` (a scene from `assemble`) turns the loss off.
    pub floor: Option<ColliderHandle>,
    /// Large impacts since the last `take_impacts`, oldest first, at most
    /// `MAX_IMPACTS` (`impacts.rs`; the page's glowing marks).
    pub impacts: Vec<Impact>,
    /// `Some` in AI vs AI: an AI drives the robot in place of the radio,
    /// on the same countdown as the opponent's.
    pub pilot: Option<AiDriver>,
}

/// How many unread impacts `Sim` keeps: a reader that never takes them
/// (a test) doesn't grow the list forever.
pub const MAX_IMPACTS: usize = 256;

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
        let floor = add_arena(&mut world);
        let robot = spawn_robot(&mut world, &model, mesh, -1.0, 0.0, 0.0)?;
        Ok(match kind {
            OpponentKind::Copy | OpponentKind::Ai | OpponentKind::AiVsAi => {
                let (o_model, o_mesh) = match rival {
                    Some(r) => (r.model, r.mesh),
                    None => (model.clone(), mesh),
                };
                let handles =
                    spawn_robot(&mut world, &o_model, o_mesh, 1.0, 0.0, std::f32::consts::PI)?;
                let ai = matches!(kind, OpponentKind::Ai | OpponentKind::AiVsAi);
                let opponent = Opponent::new(handles, o_model, ai);
                let mut s = Self::assemble(world, robot, model, Some(opponent));
                s.floor = Some(floor);
                if kind == OpponentKind::AiVsAi {
                    s.pilot = Some(AiDriver::default());
                }
                s
            }
            OpponentKind::Box => {
                let target = add_target(&mut world, 1.0, 0.0, model.total_mass() as f32);
                let mut s = Self::assemble(world, robot, model, None);
                s.target = Some(target);
                s.floor = Some(floor);
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
        // Speculative contacts within 5 mm, not the default 20 mm: at 2 kHz
        // a body at 10 m/s moves 5 mm a step.
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
            floor: None,
            impacts: Vec::new(),
            pilot: None,
        }
    }

    /// One control tick: decide duties from the radio frame (`None` = no
    /// radio) and the opponent's, then run `physics_hz() / CONTROL_HZ`
    /// physics steps with them held.
    pub fn control_tick(&mut self, frame: Option<&Frame>) {
        let dt = 1.0 / CONTROL_HZ as f32;
        let target = self.opponent_chassis_handle();
        self.duties = match (&mut self.pilot, &self.mixer) {
            // AI vs AI: held braked like the opponent until FIGHT, then
            // the AI drives, whatever the radio says.
            (Some(ai), _) => match target {
                Some(t) if self.time_s >= COUNTDOWN_S => {
                    let me = pose_of(&self.world.bodies[self.robot.bodies[0]]);
                    let weapon = weapon_fraction(&self.world, &self.robot, &self.model);
                    ai.tick(&me, &pose_of(&self.world.bodies[t]), weapon, dt)
                }
                _ => OPPONENT_DUTIES,
            },
            (None, Some(cfg)) => mix(frame, cfg),
            (None, None) => Duties::default(),
        };
        if let Some(o) = &mut self.opponent {
            o.control(&self.world, self.robot.bodies[0], self.time_s, dt);
        }
        if self.bout_left() == Some(0.0) {
            // Time: both robots get no pulses and coast down.
            self.duties = Duties::default();
            if let Some(o) = &mut self.opponent {
                o.duties = Duties::default();
            }
        }
        for _ in 0..self.physics_hz() / CONTROL_HZ {
            self.physics_step();
        }
        self.time_s += dt;
    }

    /// The physics rate, Hz.
    pub fn physics_hz(&self) -> u32 {
        (1.0 / self.world.integration_parameters.dt).round() as u32
    }

    /// Steps the physics at `hz` from the next control tick on, taken to
    /// the nearest of `PHYSICS_RATES_HZ`. Safe mid-round: the motors and
    /// the impact loss read the step from the world each step.
    pub fn set_physics_hz(&mut self, hz: u32) {
        let hz = *PHYSICS_RATES_HZ
            .iter()
            .min_by_key(|r| r.abs_diff(hz))
            .unwrap();
        self.world.integration_parameters.dt = 1.0 / hz as f32;
    }

    /// Seconds left before the AI opponent starts; `None` when there is
    /// no AI or it has started.
    pub fn countdown_left(&self) -> Option<f32> {
        let ai = self.opponent.as_ref().is_some_and(|o| o.ai.is_some());
        (ai && self.time_s < COUNTDOWN_S).then_some(COUNTDOWN_S - self.time_s)
    }

    /// Seconds left in the AI bout: all of `BOUT_S` through the
    /// countdown, then counting down from FIGHT to 0, when both robots are
    /// disarmed. `None` without an AI: the copy and the box are untimed.
    pub fn bout_left(&self) -> Option<f32> {
        let ai = self.opponent.as_ref().is_some_and(|o| o.ai.is_some());
        ai.then(|| (COUNTDOWN_S + BOUT_S - self.time_s).clamp(0.0, BOUT_S))
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
        let groups = self.floor.map(|_| self.groups());
        let before = groups.as_ref().map(|g| snapshot(&self.world, g));
        self.world.step();
        if let (Some(floor), Some(g), Some(v)) = (self.floor, &groups, &before) {
            self.impacts.extend(find_impacts(&self.world, g, floor));
            let over = self.impacts.len().saturating_sub(MAX_IMPACTS);
            self.impacts.drain(..over);
            absorb(&mut self.world, g, v, floor);
        }
        self.drive.observe(&self.world, &self.model, &self.robot);
        if let Some(o) = &mut self.opponent {
            o.drive.observe(&self.world, &o.model, &o.robot);
        }
    }

    /// The large impacts since the last call, oldest first.
    pub fn take_impacts(&mut self) -> Vec<Impact> {
        std::mem::take(&mut self.impacts)
    }

    /// Each robot (and the box) as one group for `impact_loss.rs`.
    fn groups(&self) -> Vec<Group> {
        let robot = |h: &RobotHandles, m: &RobotModel| Group {
            bodies: h.bodies.clone(),
            rotors: m
                .wheels
                .iter()
                .chain(&m.weapons)
                .map(|r| (h.bodies[r.body], v(r.axis_dir).normalize()))
                .collect(),
        };
        let mut g = vec![robot(&self.robot, &self.model)];
        if let Some(o) = &self.opponent {
            g.push(robot(&o.robot, &o.model));
        }
        if let Some(t) = self.target {
            g.push(Group {
                bodies: vec![t],
                rotors: vec![],
            });
        }
        g
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
        self.opponent_chassis_handle()
            .map(|h| &self.world.bodies[h])
    }

    fn opponent_chassis_handle(&self) -> Option<RigidBodyHandle> {
        self.opponent
            .as_ref()
            .map(|o| o.robot.bodies[0])
            .or(self.target)
    }
}
