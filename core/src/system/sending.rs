extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::serde::ser::Serialize;

use model::comp::{Pos, Vel, Connect, Disconnect, Player as PlayerComp, Actor};
use model::client::{Message as ClientMessage, OpCode};

use std::collections::HashMap;
use std::fmt::Debug;

pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, PlayerComp>,
     ReadStorage<'a, Actor>,
     WriteStorage<'a, Connect>,
     ReadStorage<'a, Disconnect>,
     Entities<'a>);

    fn run(
        &mut self,
        (pos, vel, player, actor, mut connect, disconnect, entities): Self::SystemData,
    ) {
        let mut serialized_actors = HashMap::new();
        for actor in (&actor).join() {
            serialized_actors.insert(actor.id, HashMap::new());
        }

        for (pos, vel, actor) in (&pos, &vel, &actor).join() {
            let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
            actor.insert("pos", serialize(&pos));
            actor.insert("vel", serialize(&vel));
        }

        for (player, actor) in (&player, &actor).join() {
            let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
            actor.insert("delay", serialize(&player.delay));
        }

        let mut new_connections = Vec::new();
        for (new_player, entity, new_actor, _) in
            (&player, &*entities, &actor, &mut connect).join()
        {
            new_connections.push((entity.clone(), new_actor.clone()));
            let mut payload = Vec::new();
            payload.push(serialize(&new_actor.id));
            let mut id_actor_kind = HashMap::new();
            for actor in (&actor).join() {
                id_actor_kind.insert(actor.id, actor.kind.clone());
            }
            payload.push(serialize(&id_actor_kind));
            let greeting = ClientMessage {
                opcode: OpCode::Greeting,
                payload: payload,
            };
            send(new_player, &greeting);
        }

        for new_connection in new_connections {
            let (new_entity, new_actor) = new_connection;
            connect.remove(new_entity);
            let connection = ClientMessage {
                opcode: OpCode::Connect,
                payload: new_actor.clone(),
            };
            for (player, entity) in (&player, &*entities).join() {
                if entity != new_entity {
                    send(player, &connection);
                }
            }
        }


        let mut new_disconnects = Vec::new();
        for (actor, _) in (&actor, &disconnect).join() {
            new_disconnects.push(actor.id.clone());
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
            payload: serialized_actors,
        };
        for player in (&player).join() {
            send(player, &world_state);
        }
    }
}

fn send<T>(player: &PlayerComp, msg: &ClientMessage<T>)
where
    T: Serialize,
{
    let msg = serde_json::to_string(&msg).unwrap();
    let send_channel = player.send_channel.clone();
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}

fn serialize<T>(t: &T) -> String
where
    T: Serialize + Debug,
{
    serde_json::to_string(t).expect(&format!("Failed to serialize object {:?}", t))
}
