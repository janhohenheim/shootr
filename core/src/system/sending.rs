extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities, EntitiesRes};
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

        handle_new_connections(&player, &*entities, &actor, &mut connect);
        handle_disconnects(&player, &actor, &disconnect);

        send_world_updates(&player, &actor, &pos, &vel);
    }
}


fn send<T>(player: &PlayerComp, msg: &ClientMessage<T>)
where
    T: Serialize + Debug,
{
    let msg = serde_json::to_string(&msg).expect(&format!("Failed to serialize object {:?}", msg));
    let send_channel = player.send_channel.clone();
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}


fn handle_new_connections(
    player: &ReadStorage<PlayerComp>,
    entities: &EntitiesRes,
    actor: &ReadStorage<Actor>,
    connect: &mut WriteStorage<Connect>,
) {
    let mut new_connections = Vec::new();
    for (new_player, entity, new_actor, _) in (player, entities, actor, &mut *connect).join() {
        new_connections.push((entity.clone(), new_actor.clone()));
        let mut payload = Vec::new();
        payload.push(json!({"id": new_actor.id}));
        let mut actors = Vec::new();
        for actor in (&actor).join() {
            actors.push(actor);
        }
        payload.push(json!({"actors": actors}));
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
        for (player, entity) in (player, entities).join() {
            if entity != new_entity {
                send(player, &connection);
            }
        }
    }
}

fn handle_disconnects(
    player: &ReadStorage<PlayerComp>,
    actor: &ReadStorage<Actor>,
    disconnect: &ReadStorage<Disconnect>,
) {
    for (actor, _) in (actor, disconnect).join() {
        let disconnect = ClientMessage {
            opcode: OpCode::Disconnect,
            payload: actor.id.clone(),
        };
        for player in (player).join() {
            send(player, &disconnect);
        }
    }

}

fn send_world_updates(
    player: &ReadStorage<PlayerComp>,
    actor: &ReadStorage<Actor>,
    pos: &ReadStorage<Pos>,
    vel: &ReadStorage<Vel>,
) {
    let mut serialized_actors = HashMap::new();
    for actor in (actor).join() {
        serialized_actors.insert(actor.id, Vec::new());
    }

    for (pos, vel, actor) in (pos, vel, actor).join() {
        let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
        actor.push(json!({
            "pos": pos,
            "vel": vel
        }));
    }

    for (player, actor) in (player, actor).join() {
        let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
        actor.push(json!({"delay": player.delay}));
    }
    let world_state = ClientMessage {
        opcode: OpCode::WorldUpdate,
        payload: serialized_actors,
    };
    for player in (player).join() {
        send(player, &world_state);
    }
}
