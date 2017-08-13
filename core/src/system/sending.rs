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
use util::SeqId;

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
     Fetch<'a, SeqId>,
     Entities<'a>);

    fn run(&mut self, data: Self::SystemData) {
        let (pos, vel, player, actor, mut connect, disconnect, curr_tick, entities) = data;

        handle_new_connections(&player, *curr_tick, &*entities, &actor, &mut connect);
        handle_disconnects(&player, *curr_tick, &actor, &disconnect);

        send_world_updates(&player, *curr_tick, &actor, &pos, &vel);
    }
}


fn send<T>(player: &PlayerComp, curr_tick: SeqId, msg: &ClientMessage<T>)
where
    T: Serialize + Debug,
{
    let mut msg =
        serde_json::to_string(&msg).expect(&format!("Failed to serialize object {:?}", msg));
    let send_channel = player.send_channel.clone();
    let tick = format!(",\"tick\":{}", curr_tick);
    let json_end_pos = msg.len() - 1;
    msg.insert_str(json_end_pos, &tick);
    send_channel.send(Message::Text(msg)).wait().expect(
        "Failed to send message",
    );
}


fn handle_new_connections(
    player: &ReadStorage<PlayerComp>,
    curr_tick: SeqId,
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
                send(player, curr_tick, &greeting_msg);
            } else {
                send(player, curr_tick, &other_spawn_msg);
            }
        }
    }
}

fn handle_disconnects(
    player: &ReadStorage<PlayerComp>,
    curr_tick: SeqId,
    actor: &ReadStorage<Actor>,
    disconnect: &ReadStorage<ToDespawn>,
) {
    for (actor, _) in (actor, disconnect).join() {
        let msg = ClientMessage::new_despawn(&actor.id);
        for player in (player).join() {
            send(player, curr_tick, &msg);
        }
    }

}

fn send_world_updates(
    player: &ReadStorage<PlayerComp>,
    curr_tick: SeqId,
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
        send(player, curr_tick, &world_state);
    }
}
