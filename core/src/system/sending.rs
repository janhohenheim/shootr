extern crate specs;
extern crate futures;

use self::specs::{Join, ReadStorage, Fetch, System};
use self::futures::{Future, Sink};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use model::comp::{Pos, Vel, Acc, PlayerId, Bounciness};
use model::client::{ClientState, Ball, Player};
use engine::{Id, Engine};
use util::timestamp;

type Ids = Arc<RwLock<Vec<Id>>>;

pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (Fetch<'a, Ids>,
     Fetch<'a, Engine>,
     ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, PlayerId>,
     ReadStorage<'a, Bounciness>);

    fn run(&mut self, data: Self::SystemData) {
        let (ids, engine, pos, vel, acc, id, bounciness) = data;

        let ids = ids.deref();
        let engine = engine.deref();

        let (ball_pos, ball_vel, _) = (&pos, &vel, &bounciness).join().take(1).next().unwrap();
        let ball = Ball {
            pos: ball_pos.clone(),
            vel: ball_vel.clone(),
        };

        let mut players = HashMap::new();
        for (pos, vel, acc, id) in (&pos, &vel, &acc, &id).join() {
            players.insert(
                *id.deref(),
                Player {
                    pos: pos.clone(),
                    vel: vel.clone(),
                    acc: acc.clone(),
                },
            );
        }
        let state = ClientState {
            ball,
            players,
            timestamp: timestamp(),
        };
        send(engine, ids, state);
    }
}

fn send(engine: &Engine, ids: &Ids, state: ClientState) {
    let state = Arc::new(RwLock::new(state));
    let connections = engine.connections.read().unwrap();
    for id in ids.read().unwrap().iter() {
        let channel = connections
            .get(&id)
            .expect("Didn't find connection in list")
            .write()
            .unwrap()
            .send_channel
            .clone();
        channel.send(state.clone()).wait().expect(
            "Failed to send message",
        );
    }
}
