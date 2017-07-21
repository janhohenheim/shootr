extern crate specs;
extern crate futures;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, System};
use self::futures::{Future, Sink};
use self::websocket_server::OwnedMessage;

use model::comp::{Pos, Vel, Acc, Bounciness, Player as PlayerComp};
use model::client::{ClientState, Ball, Player};

use std::collections::HashMap;


pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Vel>,
        ReadStorage<'a, Acc>,
        ReadStorage<'a, Bounciness>,
        ReadStorage<'a, PlayerComp>,
    );

    fn run(&mut self, (pos, vel, acc, bounciness, player): Self::SystemData) {
        let (ball_pos, ball_vel, _) = (&pos, &vel, &bounciness).join().take(1).next().unwrap();
        let ball = Ball {
            pos: ball_pos.clone(),
            vel: ball_vel.clone(),
        };

        let mut players = HashMap::new();
        for (pos, vel, acc, player) in (&pos, &vel, &acc, &player).join() {
            players.insert(
                player.id,
                Player {
                    delay: player.delay,
                    pos: pos.clone(),
                    vel: vel.clone(),
                    acc: acc.clone(),
                },
            );
        }
        let state = ClientState { ball, players };

        for player in &mut player.join() {
            send(player, &state);
        }
    }
}

fn send(player: &PlayerComp, state: &ClientState) {
    let msg = serde_json::to_string(&state).unwrap();
    let send_channel = player.send_channel.clone();
    send_channel
        .send(OwnedMessage::Text(msg))
        .wait()
        .expect("Failed to send message");
}
