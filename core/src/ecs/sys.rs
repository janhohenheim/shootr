extern crate specs;
extern crate futures;

use self::specs::{Join, ReadStorage, WriteStorage, Fetch, System};

use super::comp::{Pos, Vel};
use super::res::TimeProgess;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use engine::{Engine, spawn_future};
use model::ClientState;

use self::futures::{Future, BoxFuture, Sink};
use self::futures::future::{self, Loop};
use self::futures::sync::mpsc;

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

pub struct Send;
impl<'a> System<'a> for Send {
    type SystemData = (Fetch<'a, TimeProgess>, Fetch<'a, Engine>, ReadStorage<'a, Pos>);

    fn run(&mut self, data: Self::SystemData) {
        let (progress, engine, pos) = data;
        let progress = progress.deref();
        let engine = engine.deref();
        for pos in pos.join() {
            println!("Pos: {:?}", pos);
        }
        let state = ClientState {};
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
