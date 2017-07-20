extern crate specs;
extern crate futures;
extern crate serde_json;

use self::specs::{Join, ReadStorage, System};
use self::futures::{Future, Sink};
use std::collections::HashMap;

use model::comp::{Pos, Vel, Acc, Bounciness, Player as PlayerComp};
use model::client::{ClientState, Ball, Player};
use engine::{OwnedMessage, SendChannel};
use util::timestamp;


pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, Bounciness>,
     ReadStorage<'a, PlayerComp>);

    fn run(&mut self, (pos, vel, acc, bounciness, player): Self::SystemData) {
        let (ball_pos, ball_vel, _) = (&pos, &vel, &bounciness).join().take(1).next().unwrap();
        let ball = Ball {
            pos: ball_pos.clone(),
            vel: ball_vel.clone(),
        };

        let mut players = HashMap::new();
        for (pos, vel, acc) in (&pos, &vel, &acc).join() {
            players.insert(
                "Foo".to_owned(),
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
        for player in &mut player.join() {
            send(player.send_channel.clone(), &state);
        }
    }
}

fn send(send_channel: SendChannel, state: &ClientState) {
    let msg = serde_json::to_string(&state).unwrap();
    send_channel.send(OwnedMessage::Text(msg)).wait().expect(
        "Failed to send message",
    );
}
