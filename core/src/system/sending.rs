extern crate specs;
extern crate futures;
extern crate serde;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, ReadStorage, WriteStorage, System, Entities, EntitiesRes, Fetch};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::serde::ser::Serialize;

use model::comp::{Pos, Vel, ToSpawn, ToDespawn, Player as PlayerComp, Actor};
use model::client::{Message as ClientMessage, OpCode};
use util::StopWatch;

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;

pub struct Sending;
impl<'a> System<'a> for Sending {
    #[allow(type_complexity)]
    type SystemData = (ReadStorage<'a, Pos>,
     ReadStorage<'a, Vel>,
     ReadStorage<'a, PlayerComp>,
     ReadStorage<'a, Actor>,
     WriteStorage<'a, ToSpawn>,
     ReadStorage<'a, ToDespawn>,
     Fetch<'a, RwLock<StopWatch>>,
     Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (pos, vel, player, actor, mut connect, disconnect, server_clock, entities) = data;
        let server_clock = server_clock.read().unwrap();

        handle_new_connections(&player, &server_clock, &*entities, &actor, &mut connect);
        handle_disconnects(&player, &server_clock, &actor, &disconnect);

        send_world_updates(&player, &server_clock, &actor, &pos, &vel);
    }
}


fn send<T>(player: &PlayerComp, server_clock: &StopWatch, msg: &ClientMessage<T>)
where
    T: Serialize + Debug,
{
    let mut msg =
        serde_json::to_string(&msg).expect(&format!("Failed to serialize object {:?}", msg));
    let send_channel = player.send_channel.clone();
    let timestamp = format!(",\"server_time\":{}", server_clock.get_ms());
    let json_end_pos = msg.len() - 1;
    msg.insert_str(json_end_pos, &timestamp);
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}


fn handle_new_connections(
    player: &ReadStorage<PlayerComp>,
    server_clock: &StopWatch,
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
                send(player, server_clock, &greeting_msg);
            } else {
                send(player, server_clock, &other_spawn_msg);
            }
        }
    }
}

fn handle_disconnects(
    player: &ReadStorage<PlayerComp>,
    server_clock: &StopWatch,
    actor: &ReadStorage<Actor>,
    disconnect: &ReadStorage<ToDespawn>,
) {
    for (actor, _) in (actor, disconnect).join() {
        let msg = ClientMessage::new_despawn(&actor.id);
        for player in (player).join() {
            send(player, server_clock, &msg);
        }
    }

}

fn send_world_updates(
    player: &ReadStorage<PlayerComp>,
    server_clock: &StopWatch,
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
        send(player, server_clock, &world_state);
    }
}
