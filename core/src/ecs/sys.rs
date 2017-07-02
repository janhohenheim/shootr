extern crate specs;
extern crate futures;

use self::specs::{Join, ReadStorage, WriteStorage, Fetch, System};

use super::comp::{Pos, Vel};
use super::res::TimeProgress;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use engine::{Engine, spawn_future};
use model::{ClientState, Axis};

use self::futures::{Future, Sink};

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Vel>);

    fn run(&mut self, (mut pos, mut vel): Self::SystemData) {
        let min = Pos { x: 0, y: 0 };
        let max = Pos { x: 1000, y: 1000 };
        for (mut pos, mut vel) in (&mut pos, &mut vel).join() {
            handle_movement(&mut pos, &mut vel, &min, &max);
        }
    }
}

fn handle_movement(pos: &mut Pos, vel: &mut Vel, min: &Pos, max: &Pos) {
    let new_x = pos.x + vel.x;
    let new_y = pos.y + vel.y;
    if new_x > max.x {
        bounce(pos, vel, max, &Axis::X);
    } else if new_x < 0 {
        bounce(pos, vel, min, &Axis::X);
    } else if new_y > max.y {
        bounce(pos, vel, max, &Axis::Y);
    } else if new_y < 0 {
        bounce(pos, vel, min, &Axis::Y);
    } else {
        pos.x = new_x;
        pos.y = new_y;
    }
}

fn bounce(pos: &mut Pos, vel: &mut Vel, max: &Pos, overflowing_axis: &Axis) {
    let (pos_overflow, pos_other) = match *overflowing_axis {
        Axis::X => (&mut pos.x, &mut pos.y),
        Axis::Y => (&mut pos.y, &mut pos.x),
    };
    let (vel_overflow, vel_other) = match *overflowing_axis {
        Axis::X => (&mut vel.x, &mut vel.y),
        Axis::Y => (&mut vel.y, &mut vel.x),
    };
    let (max_overflow, _) = match *overflowing_axis {
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
        send(engine, state);
    }
}

fn send(engine: &Engine, state: ClientState) {
    let state = Arc::new(RwLock::new(state));
    for (id, _) in engine.connections.read().unwrap().iter() {
        let channel = engine.send_channel.clone();
        channel.send((*id, state.clone())).wait().unwrap();
    }
}
