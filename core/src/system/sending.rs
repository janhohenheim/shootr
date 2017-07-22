extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::serde::ser::Serialize;

use model::comp::{Pos, Vel, Acc, Bounciness, Connect, Disconnect, Player as PlayerComp};
use model::client::{ClientState, Ball, Player, Greeting, ConnectionInfo, ConnectionStatus};


pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, Bounciness>,
     ReadStorage<'a, PlayerComp>,
     WriteStorage<'a, Connect>,
     ReadStorage<'a, Disconnect>,
     Entities<'a>);

    fn run(
        &mut self,
        (pos, vel, acc, bounciness, player, mut connect, disconnect, entities): Self::SystemData,
    ) {
        let (ball_pos, ball_vel, _) = (&pos, &vel, &bounciness).join().take(1).next().unwrap();
        let ball = Ball {
            pos: ball_pos.clone(),
            vel: ball_vel.clone(),
        };

        let mut players = Vec::new();
        for (pos, vel, acc, player) in (&pos, &vel, &acc, &player).join() {
            players.push(Player {
                id: player.id,
                delay: player.delay,
                pos: pos.clone(),
                vel: vel.clone(),
                acc: acc.clone(),
            });
        }
        let state = ClientState {
            ball: ball.clone(),
            players: players.clone(),
        };

        let mut new_connections = Vec::new();
        for (player, entity, _) in (&player, &*entities, &mut connect).join() {
            new_connections.push((entity.clone(), player.id.clone()));
            let greeting = Greeting {
                client_id: player.id,
                ball: ball.clone(),
                players: players.clone(),
            };

            send(player, &greeting);
        }

        for new_connection in new_connections {
            let (new_entity, new_id) = new_connection;
            connect.remove(new_entity);
            let connection = ConnectionInfo {
                player_id: new_id,
                status: ConnectionStatus::Connected,
            };
            for (player, entity) in (&player, &*entities).join() {
                if entity != new_entity {
                    send(player, &connection);
                }
            }
        }


        let mut new_disconnects = Vec::new();
        for (player, _) in (&player, &disconnect).join() {
            new_disconnects.push(player.id);
        }

        for new_disconnect in new_disconnects {
            let disconnect = ConnectionInfo {
                player_id: new_disconnect,
                status: ConnectionStatus::Disconnected,
            };
            for player in (&player).join() {
                send(player, &disconnect);
            }
        }

        for player in (&player).join() {
            send(player, &state);
        }
    }
}

fn send<T>(player: &PlayerComp, state: &T)
where
    T: Serialize,
{
    let msg = serde_json::to_string(&state).unwrap();
    let send_channel = player.send_channel.clone();
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}
