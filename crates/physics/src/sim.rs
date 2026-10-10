//! The fixed-step loop: physics at `PHYSICS_HZ`, control at `CONTROL_HZ`
//! (docs/drivesim.md §6: sized from the fastest rotor; a 10,000 rpm
//! weapon is unstable on a rapier joint at 1 ms and stable from 2 kHz).

use crate::arena::{add_arena, add_target};
use crate::gear_coupling::gear_coupling;
use crate::mixer::{Duties, MixerConfig, mix};
use crate::motor::rotor_torque;
use crate::spawn::{RobotHandles, SpawnError, forward_sign, side_of, spawn_robot, v};
use drivesim_input::Frame;
use drivesim_robot::robot_file::Control;
use drivesim_robot::{RobotModel, Rotor, Side};
use rapier3d::prelude::*;

pub const PHYSICS_HZ: u32 = 4000;
pub const CONTROL_HZ: u32 = 1000;
pub const STEPS_PER_TICK: u32 = PHYSICS_HZ / CONTROL_HZ;
pub const GRAVITY: f32 = 9.81;

pub struct Sim {
    pub world: PhysicsWorld,
    pub robot: RobotHandles,
    pub model: RobotModel,
    pub target: RigidBodyHandle,
    mixer: Option<MixerConfig>,
    pub duties: Duties,
    /// Per rotor (wheels, then weapons): (rotor, chassis) absolute angular
    /// velocity about the rotor's axis after the last step, and the
    /// accelerations over it, for `gear_coupling`.
    axis_w: Vec<(f32, f32)>,
    axis_alpha: Vec<(f32, f32)>,
}

impl Sim {
    /// The arena, the target box at (1, 0) and the robot at (−1, 0) facing
    /// it.
    pub fn new(
        model: RobotModel,
        mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> Result<Sim, SpawnError> {
        let mut world = Self::empty_world(-GRAVITY);
        let _floor = add_arena(&mut world);
        let target = add_target(&mut world, 1.0, 0.0, model.total_mass() as f32);
        let robot = spawn_robot(&mut world, &model, mesh, -1.0, 0.0, 0.0)?;
        Ok(Self::assemble(world, robot, model, target))
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
        target: RigidBodyHandle,
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
            target,
            mixer,
            duties: Duties::default(),
            axis_w: Vec::new(),
            axis_alpha: Vec::new(),
        }
    }

    fn rotors(&self) -> impl Iterator<Item = &Rotor> {
        self.model.wheels.iter().chain(&self.model.weapons)
    }

    /// (rotor, chassis) absolute angular velocity about each rotor's axis.
    fn axis_speeds(&self) -> Vec<(f32, f32)> {
        let c = &self.world.bodies[self.robot.bodies[0]];
        self.rotors()
            .map(|r| {
                let a = *c.rotation() * v(r.axis_dir).normalize();
                let b = &self.world.bodies[self.robot.bodies[r.body]];
                (b.angvel().dot(a), c.angvel().dot(a))
            })
            .collect()
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

    /// One physics step with the current duties: motor torques from each
    /// rotor's present speed, applied equal and opposite to rotor and
    /// chassis about the joint axis.
    pub fn physics_step(&mut self) {
        let chassis = self.robot.bodies[0];
        for &h in &self.robot.bodies {
            self.world.bodies[h].reset_torques(false);
        }
        let v_batt = self.model.battery_v as f32;
        let mut torques = Vec::new();
        for wh in &self.model.wheels {
            let duty = match side_of(wh) {
                Side::Left => self.duties.left,
                Side::Right => self.duties.right,
            };
            let s = forward_sign(wh);
            let w = s * self.rotor_speed(wh);
            torques.push((
                wh.body,
                s * rotor_torque(duty, w, wh, v_batt),
                v(wh.axis_dir),
            ));
        }
        for wp in &self.model.weapons {
            let w = self.rotor_speed(wp);
            torques.push((
                wp.body,
                rotor_torque(self.duties.weapon, w, wp, v_batt),
                v(wp.axis_dir),
            ));
        }
        let rot = *self.world.bodies[chassis].rotation();
        for (k, (body, tau, axis)) in torques.into_iter().enumerate() {
            let rotor = self.rotors().nth(k).expect("one torque per rotor");
            let (alpha_r, alpha_c) = self.axis_alpha.get(k).copied().unwrap_or_default();
            let (fix_r, fix_c) = gear_coupling(
                rotor.motor.rotor_inertia_kg_m2 as f32,
                rotor.ratio as f32,
                alpha_r,
                alpha_c,
            );
            let a = rot * axis.normalize();
            let h = self.robot.bodies[body];
            self.world.bodies[h].add_torque(a * (tau + fix_r), true);
            self.world.bodies[chassis].add_torque(a * (fix_c - tau), true);
        }
        let before = if self.axis_w.is_empty() {
            self.axis_speeds()
        } else {
            std::mem::take(&mut self.axis_w)
        };
        self.world.step();
        let after = self.axis_speeds();
        let dt = self.world.integration_parameters.dt;
        self.axis_alpha = before
            .iter()
            .zip(&after)
            .map(|(b, a)| ((a.0 - b.0) / dt, (a.1 - b.1) / dt))
            .collect();
        self.axis_w = after;
    }

    /// Rotor speed relative to the chassis about its axis, rad/s.
    pub fn rotor_speed(&self, rotor: &Rotor) -> f32 {
        let c = &self.world.bodies[self.robot.bodies[0]];
        let b = &self.world.bodies[self.robot.bodies[rotor.body]];
        let a = *c.rotation() * v(rotor.axis_dir).normalize();
        (b.angvel() - c.angvel()).dot(a)
    }

    pub fn chassis(&self) -> &RigidBody {
        &self.world.bodies[self.robot.bodies[0]]
    }
}
