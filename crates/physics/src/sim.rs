//! The fixed-step loop: physics at `PHYSICS_HZ`, control at `CONTROL_HZ`
//! (docs/drivesim.md §6: sized from the fastest rotor; a 10,000 rpm
//! weapon is unstable on a rapier joint at 1 ms and stable from 2 kHz).

use crate::arena::{add_arena, add_target};
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
}

impl Sim {
    /// The arena, the target box at (1, 0) and the robot at (−1, 0) facing
    /// it.
    pub fn new(
        model: RobotModel,
        mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> Result<Sim, SpawnError> {
        let mut world = Self::empty_world(-GRAVITY);
        add_arena(&mut world);
        let target = add_target(&mut world, 1.0, 0.0);
        let robot = spawn_robot(&mut world, &model, mesh, -1.0, 0.0, 0.0)?;
        Ok(Self::assemble(world, robot, model, target))
    }

    /// A world with nothing in it, for tests that build their own scene.
    pub fn empty_world(gravity_z: f32) -> PhysicsWorld {
        let mut w = PhysicsWorld::new();
        w.gravity = Vector::new(0.0, 0.0, gravity_z);
        w.integration_parameters.dt = 1.0 / PHYSICS_HZ as f32;
        w.integration_parameters.num_solver_iterations = 2;
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
        for (body, tau, axis) in torques {
            let a = rot * axis.normalize();
            let h = self.robot.bodies[body];
            self.world.bodies[h].add_torque(a * tau, true);
            self.world.bodies[chassis].add_torque(-a * tau, true);
        }
        self.world.step();
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
