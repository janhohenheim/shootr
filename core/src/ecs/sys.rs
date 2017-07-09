extern crate specs;
extern crate futures;

use self::specs::{Join, ReadStorage, WriteStorage, Fetch, System};
use self::futures::{Future, Sink};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use super::comp::{Pos, Vel};
use super::res::Ids;
use engine::Engine;
use model::ClientState;


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
    pos.x += vel.x;
    pos.y += vel.y;
    if pos.x > max.x {
        vel.x = -vel.x;
        pos.x = max.x;
    }
    if pos.y > max.y {
        vel.y = -vel.y;
        pos.y = max.y;
    }
    if pos.x < min.x {
        vel.x = -vel.x;
        pos.x = min.x;
    }
    if pos.y < min.y {
        vel.y = -vel.y;
        pos.y = min.y;
    }
}


pub struct Send;
impl<'a> System<'a> for Send {
    type SystemData = (Fetch<'a, Ids>, Fetch<'a, Engine>, ReadStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, data: Self::SystemData) {
        let (ids, engine, pos, vel) = data;
        let ids = ids.deref();
        let engine = engine.deref();

        let (pos, vel) = (&pos, &vel).join().take(1).next().unwrap();
        let state = ClientState {
            pos: pos.clone(),
            vel: vel.clone(),
        };
        send(engine, ids, state);
    }
}

fn send(engine: &Engine, ids: &Ids, state: ClientState) {
    let state = Arc::new(RwLock::new(state));
    for id in ids.read().unwrap().iter() {
        let channel = engine.send_channel.clone();
        channel.send((*id, state.clone())).wait().unwrap();
    }
}
