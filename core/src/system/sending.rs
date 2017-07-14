extern crate specs;
extern crate futures;
extern crate chrono;

use self::specs::{Join, ReadStorage, Fetch, System};
use self::futures::{Future, Sink};
use self::chrono::{TimeZone, Utc};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use model::comp::{Pos, Vel, Acc, PlayerInput, Bounciness};
use model::client::{ClientState, Ball, Player};
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
        ReadStorage<'a, Acc>,
        ReadStorage<'a, PlayerInput>,
        ReadStorage<'a, Bounciness>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (ids, engine, pos, vel, acc, player_input, bounciness) = data;

        let ids = ids.deref();
        let engine = engine.deref();

        let (ball_pos, ball_vel, _) = (&pos, &vel, &bounciness).join().take(1).next().unwrap();
        let ball = Ball {
            pos: ball_pos.clone(),
            vel: ball_vel.clone(),
        };
        let (player_pos, player_vel, player_acc, _) = (&pos, &vel, &acc, &player_input)
            .join()
            .take(1)
            .next()
            .unwrap();
        let player = Player {
            pos: player_pos.clone(),
            vel: player_vel.clone(),
            acc: player_acc.clone(),
        };
        let state = ClientState {
            ball,
            player,
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
