extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities, EntitiesRes};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::serde::ser::Serialize;

use model::comp::{Pos, Vel, ToSpawn, ToDespawn, Player as PlayerComp, Actor};
use model::client::{Message as ClientMessage, OpCode};
use util::timestamp;

use std::collections::HashMap;
use std::fmt::Debug;

pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, PlayerComp>,
     ReadStorage<'a, Actor>,
     WriteStorage<'a, ToSpawn>,
     ReadStorage<'a, ToDespawn>,
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
    let mut msg =
        serde_json::to_string(&msg).expect(&format!("Failed to serialize object {:?}", msg));
    let send_channel = player.send_channel.clone();
    let timestamp = format!(",\"timestamp\":{}", timestamp());
    let json_end_pos = msg.len() - 1;
    msg.insert_str(json_end_pos, &timestamp);
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}


fn handle_new_connections(
    player: &ReadStorage<PlayerComp>,
    entities: &EntitiesRes,
    actor: &ReadStorage<Actor>,
    spawn: &mut WriteStorage<ToSpawn>,
) {
    let mut new_connections = Vec::new();
    for (entity, actor, _) in (entities, actor, &mut *spawn).join() {
        new_connections.push((entity, actor.clone()));
    }

    let mut actors = Vec::new();
    for actor in (actor).join() {
        actors.push(actor);
    }
    for new_connection in new_connections {
        let (new_entity, new_actor) = new_connection;
        spawn.remove(new_entity);
        let greeting_msg = ClientMessage::new_greeting(&new_actor.id, &actors);
        let other_spawn_msg = ClientMessage::new_spawn(&new_actor);
        for (player, entity) in (player, entities).join() {
            if entity == new_entity {
                send(player, &greeting_msg);
            } else {
                send(player, &other_spawn_msg);
            }
        }
    }
}

fn handle_disconnects(
    player: &ReadStorage<PlayerComp>,
    actor: &ReadStorage<Actor>,
    disconnect: &ReadStorage<ToDespawn>,
) {
    for (actor, _) in (actor, disconnect).join() {
        let msg = ClientMessage::new_despawn(&actor.id);
        for player in (player).join() {
            send(player, &msg);
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
        serialized_actors.insert(actor.id, HashMap::new());
    }

    for (pos, vel, actor) in (pos, vel, actor).join() {
        let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
        actor.insert("pos", json!(pos));
        actor.insert("vel", json!(vel));
    }

    for (player, actor) in (player, actor).join() {
        let mut actor = serialized_actors.get_mut(&actor.id).unwrap();
        actor.insert("delay", json!(player.delay));
    }
    let json_actors = json!(serialized_actors);
    for player in (player).join() {
        let last_input = json!(player.last_input);
        let payload =
            hashmap!(
            "last_input" => &last_input,
            "actors" => &json_actors
        );
        let world_state = ClientMessage {
            opcode: OpCode::WorldUpdate,
            payload: &payload,
        };
        send(player, &world_state);
    }
}
