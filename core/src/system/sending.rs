extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::serde::ser::Serialize;

use model::comp::{Pos, Vel, Acc, Bounciness, Connect, Disconnect, Player as PlayerComp, WorldId};
use model::client::{Message as ClientMessage, OpCode};

use std::collections::HashMap;
use std::fmt::Debug;

pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Vel>,
        ReadStorage<'a, Acc>,
        ReadStorage<'a, Bounciness>,
        ReadStorage<'a, PlayerComp>,
        ReadStorage<'a, WorldId>,
        WriteStorage<'a, Connect>,
        ReadStorage<'a, Disconnect>,
        Entities<'a>,
    );

    fn run(
        &mut self,
        (pos, vel, acc, bounciness, player, world_id, mut connect, disconnect, entities): Self::SystemData,
    ) {
        let mut actors = HashMap::new();
        for id in (&world_id).join() {
            actors.insert(id, HashMap::new());
        }

        for (pos, vel, id) in (&pos, &vel, &world_id).join() {
            let mut actor = actors.get_mut(&id).unwrap();
            actor.insert("pos", serialize(&pos));
            actor.insert("vel", serialize(&vel));
        }


        for (player, id) in (&player, &world_id).join() {
            let mut actor = actors.get_mut(&id).unwrap();
            actor.insert("delay", serialize(&player.delay));
        }

        let mut new_connections = Vec::new();
        for (player, entity, id, _) in (&player, &*entities, &world_id, &mut connect).join() {
            new_connections.push((entity.clone(), id.clone()));
            let greeting = ClientMessage {
                opcode: OpCode::Greeting,
                payload: id.clone(),
            };
            send(player, &greeting);
        }

        for new_connection in new_connections {
            let (new_entity, new_id) = new_connection;
            connect.remove(new_entity);
            let connection = ClientMessage {
                opcode: OpCode::Connect,
                payload: new_id.clone(),
            };
            for (player, entity) in (&player, &*entities).join() {
                if entity != new_entity {
                    send(player, &connection);
                }
            }
        }


        let mut new_disconnects = Vec::new();
        for (id,  _) in (&world_id, &disconnect).join() {
            new_disconnects.push(id);
        }

        for new_disconnect in new_disconnects {
            let disconnect = ClientMessage {
                opcode: OpCode::Disconnect,
                payload: new_disconnect.clone(),
            };
            for player in (&player).join() {
                send(player, &disconnect);
            }
        }

        let world_state = ClientMessage {
            opcode: OpCode::WorldUpdate,
            payload: actors,
        };
        for player in (&player).join() {
            send(player, &world_state);
        }
    }
}

fn send<T>(player: &PlayerComp, msg: &ClientMessage<T>)
where T: Serialize
{
    let msg = serde_json::to_string(&msg).unwrap();
    let send_channel = player.send_channel.clone();
    send_channel
        .send(Message::Text(msg))
        .wait()
        .expect("Failed to send message");
}

fn serialize<T>(t: &T) -> String where T: Serialize + Debug {
    serde_json::to_string(t).expect(&format!("Failed to serialize object {:?}", t))
}