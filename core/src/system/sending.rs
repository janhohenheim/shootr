extern crate specs;
extern crate futures;
extern crate chrono;

use self::specs::{Join, ReadStorage, Fetch, System};
use self::futures::{Future, Sink};
use self::chrono::{TimeZone, Utc};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use model::comp::{Pos, Vel};
use model::client::ClientState;
use engine::{Id, Engine};
use util::elapsed_ms;

type Ids = Arc<RwLock<Vec<Id>>>;

pub struct Sending;
impl<'a> System<'a> for Sending {
    type SystemData = (
        Fetch<'a, Ids>,
        Fetch<'a, Engine>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Vel>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ids, engine, pos, vel) = data;
        let ids = ids.deref();
        let engine = engine.deref();

        let (pos, vel) = (&pos, &vel).join().take(1).next().unwrap();
        let state = ClientState {
            pos: pos.clone(),
            vel: vel.clone(),
            timestamp: elapsed_ms(Utc.timestamp(0, 0), Utc::now()),
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
