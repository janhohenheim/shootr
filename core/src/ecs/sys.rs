extern crate specs;
extern crate futures;

use self::specs::{Join, ReadStorage, WriteStorage, Fetch, System};

use super::comp::{Pos, Vel};
use super::res::TimeProgress;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use engine::{Engine, spawn_future};
use model::{ClientState, Axis};

use self::futures::Sink;

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Vel>);

    fn run(&mut self, (mut pos, mut vel): Self::SystemData) {
        let max = Pos {
            x: 1000,
            y: 1000,
        };
        for (mut pos, mut vel) in (&mut pos, &mut vel).join() { 
            handle_movement(&mut pos, &mut vel, &max, &Axis::X);
            handle_movement(&mut pos, &mut vel, &max, &Axis::Y);
        }
    }
}

fn handle_movement(pos: &mut Pos, vel: &mut Vel, max: &Pos, overflowing_axis: &Axis) {
    let (mut axis_pos, mut axis_vel, mut axis_max) = match *overflowing_axis {
        Axis::X => (pos.x, vel.x, max.x),
        Axis::Y => (pos.y, vel.y, max.y),
    };
    let new_pos = axis_pos + axis_vel;
    if new_pos > axis_max {                
        bounce(pos, vel, max, overflowing_axis);
    } else if new_pos < 0 {
        bounce(pos, vel, max, overflowing_axis);
    }
    else {
        axis_pos = new_pos;
    }
}

fn bounce(pos: &mut Pos, vel: &mut Vel, max: &Pos, overflowing_axis: &Axis) {
    let (mut pos_overflow, mut pos_other) = match *overflowing_axis {
        Axis::X => (&mut pos.x, &mut pos.y),
        Axis::Y => (&mut pos.y, &mut pos.x),
    };
    let (mut vel_overflow, mut vel_other) = match *overflowing_axis {
        Axis::X => (&mut vel.x, &mut vel.y),
        Axis::Y => (&mut vel.y, &mut vel.x),
    };
    let (mut max_overflow, mut max_other) = match *overflowing_axis {
        Axis::X => (max.x, max.y),
        Axis::Y => (max.y, max.x),
    };

    let delta = (*pos_overflow + *vel_overflow) - max_overflow;
    let coefficient = *vel_overflow as f64 / delta as f64;
    *pos_overflow = max_overflow;
    *pos_other += (*vel_other as f64 * coefficient) as i32; 
    *vel_overflow = -*vel_overflow;
}

pub struct Send;
impl<'a> System<'a> for Send {
    type SystemData = (Fetch<'a, TimeProgress>, Fetch<'a, Engine>, ReadStorage<'a, Pos>);

    fn run(&mut self, data: Self::SystemData) {
        let (progress, engine, pos) = data;
        let progress = progress.deref();
        let engine = engine.deref();
    
        let state = ClientState {
            pos: pos.join().take(1).next().unwrap().clone(),
            progress: progress.clone(),
        };
        send(engine.clone(), state);
    }
}

fn send(engine: Engine, state: ClientState) {
    let state = Arc::new(RwLock::new(state));
    let engine_inner = engine.clone();
    engine.remote.spawn(move |handle| {
        let engine = engine_inner;
        for (id, _) in engine.connections.read().unwrap().iter() {
            let f = engine.channel.clone().send((*id, state.clone()));
            spawn_future(f, "Send message to write handler", handle);
        }
        Ok(())
    });
}
