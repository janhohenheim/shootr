extern crate specs;
extern crate futures;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities};
use self::futures::{Future, Sink};
use self::websocket_server::OwnedMessage;

use model::game::Id;
use model::comp::{Pos, Vel, Acc, Bounciness, Ping, Player as PlayerComp};
use model::client::{ClientState, Ball, Player};
use util::timestamp;

use std::collections::HashMap;


pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, Bounciness>,
     WriteStorage<'a, PlayerComp>,
     WriteStorage<'a, Ping>,
     Entities<'a>);

    fn run(
        &mut self,
        (pos, vel, acc, bounciness, mut player, mut ping, entities): Self::SystemData,
    ) {
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
            send(player, &state);
        }

        let mut pinged_players = Vec::new();
        for (mut player, entity, _) in (&mut player, &*entities, &mut ping).join() {

            send_ping(&mut player);
            pinged_players.push(entity);
        }

        for pinged_player in pinged_players {
            ping.remove(pinged_player);
        }
    }
}

fn send(player: &PlayerComp, state: &ClientState) {
    let msg = serde_json::to_string(&state).unwrap();
    let send_channel = player.send_channel.clone();
    send_channel.send(OwnedMessage::Text(msg)).wait().expect(
        "Failed to send message",
    );
}

fn send_ping(player: &mut PlayerComp) {
    let send_channel = player.send_channel.clone();
    let timestamp = timestamp();
    let id = Id::new_v4();
    player.pingpongs.insert(id, (timestamp, None));
    send_channel
        .send(OwnedMessage::Ping(id.as_bytes().to_vec()))
        .wait()
        .unwrap();
}
