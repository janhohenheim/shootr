extern crate shootr;

extern crate specs;
extern crate chrono;
extern crate serde_json;
extern crate websocket_server;
extern crate dotenv;

use specs::{DispatcherBuilder, World, Entity};
use chrono::prelude::*;
use websocket_server::{start as start_server, EventHandler, SendChannel, Message};
use dotenv::dotenv;

use shootr::util::{read_env_var, elapsed_ms, timestamp};
use shootr::model::comp::{Vel, Pos, Bounciness, Connect, Disconnect, Player, Ping, Pong};
use shootr::model::client::KeyState;
use shootr::model::game::{Vector, Id};
use shootr::system::*;
use shootr::bootstrap;

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

fn main() {
    dotenv().ok();
    let port = read_env_var("CORE_PORT").parse::<u32>().expect(
        "Specified port is not a valid number",
    );
    start_server::<Handler>("localhost", port);
}

struct Handler {
    id_entity: RwLock<HashMap<Id, Entity>>,
    to_spawn: RwLock<HashMap<Id, SendChannel>>,
    to_despawn: RwLock<HashSet<Id>>,
    inputs: Arc<RwLock<HashMap<Id, Vec<KeyState>>>>,
    pongs: Arc<RwLock<Vec<(Id, Id, u64)>>>,
}

impl Handler {
    fn prepare_world(&self, world: &mut World) {
        bootstrap::prepare_world(world);
        world.add_resource(self.inputs.clone());

        // Ball
        world
            .create_entity()
            .with(Vel::from(Vector { x: 15, y: 10 }))
            .with(Pos::from(Vector { x: 500, y: 500 }))
            .with(Bounciness {})
            .build();
    }


    fn handle_text(&self, id: Id, msg: &str) {
        if let Ok(key_state) = serde_json::from_str::<KeyState>(&msg) {
            let mut inputs = self.inputs.write().unwrap();
            let has_already_inputs = inputs.get(&id).is_some();
            if has_already_inputs {
                inputs.get_mut(&id).unwrap().push(key_state);
            } else {
                inputs.insert(id, vec![key_state]);
            }
        } else {
            println!("Client {}: Sent invalid message: {}", id, msg);
        }
    }

    fn handle_pong(&self, id: Id, data: &Vec<u8>) {
        let timestamp = timestamp();
        if let Ok(data) = Id::from_bytes(&data) {
            let mut pongs = self.pongs.write().unwrap();
            pongs.push((id, data, timestamp));
        } else {
            println!("Client {}: Sent pong with invalid bytes {:?}", id, data);
        }
    }


    fn register_spawns(&self, world: &mut World) {
        let mut id_entity = self.id_entity.write().unwrap();
        let mut to_spawn = self.to_spawn.write().unwrap();
        for (id, send_channel) in to_spawn.drain() {
            let entity = world
                .create_entity()
                .with(Connect {})
                .with(Player::new(id, send_channel))
                .build();
            id_entity.insert(id, entity.clone());
        }

        let mut to_despawn = self.to_despawn.write().unwrap();
        for id in to_despawn.drain() {
            if let Some(entity) = id_entity.remove(&id) {
                world.write::<Disconnect>().insert(entity, Disconnect {});
            }
        }
    }

    fn register_pings(&self, world: &mut World) {
        for (_, entity) in self.id_entity.read().unwrap().iter() {
            world.write::<Ping>().insert(entity.clone(), Ping {});
        }
    }

    fn register_pongs(&self, world: &mut World) {
        let id_entity = self.id_entity.read().unwrap();
        let mut pongs = self.pongs.write().unwrap();
        for pong in pongs.drain(..) {
            let (player_id, ping_id, timestamp) = pong;
            let entity = id_entity.get(&player_id).expect(
                "Processed pong from player that isn't in list",
            );
            world.write::<Pong>().insert(
                entity.clone(),
                Pong { ping_id, timestamp },
            );
        }
    }
}

impl EventHandler for Handler {
    type Id = Id;

    fn new() -> Self {
        Handler {
            id_entity: RwLock::new(HashMap::new()),
            to_spawn: RwLock::new(HashMap::new()),
            to_despawn: RwLock::new(HashSet::new()),
            inputs: Arc::new(RwLock::new(HashMap::new())),
            pongs: Arc::new(RwLock::new(Vec::new())),
        }
    }
    fn main_loop(&self) {
        let mut world = World::new();
        self.prepare_world(&mut world);

        let mut updater = DispatcherBuilder::new()
            .add(InputHandler, "input_handler", &[])
            .add(Physics, "physics", &["input_handler"])
            .add(Bounce, "bounce", &["physics"])
            .add(Spawn, "spawn", &["physics", "bounce"])
            .build();
        let mut sender = DispatcherBuilder::new()
            .add(Sending, "sending", &[])
            .build();
        let mut delay = DispatcherBuilder::new().add(Delay, "delay", &[]).build();

        let mut lag: u64 = 0;
        let mut previous = Utc::now();
        let updates_per_sec = read_env_var("CORE_UPDATES_PER_SEC").parse::<u64>().expect(
            "Failed to parse environmental variable as integer",
        );
        let ms_per_update = 1000 / updates_per_sec;
        let mut ping_timer = 0;
        let ping_interval = 1000;
        loop {
            let current = Utc::now();
            let elapsed = elapsed_ms(previous, current).expect("Time went backwards");
            previous = current;
            lag += elapsed;
            ping_timer += elapsed;

            if ping_timer > ping_interval {
                self.register_pings(&mut world);
                delay.dispatch(&mut world.res);
                ping_timer = 0;
            }
            self.register_pongs(&mut world);
            self.register_spawns(&mut world);

            while lag >= ms_per_update {
                updater.dispatch(&mut world.res);
                lag -= ms_per_update;
            }
            sender.dispatch(&mut world.res);
            world.maintain();

            sleep(Duration::from_millis(ms_per_update - lag));
        }
    }

    fn on_message(&self, id: Self::Id, msg: Message) {
        match msg {
            Message::Text(ref txt) => self.handle_text(id, txt),
            Message::Pong(ref data) => self.handle_pong(id, data),
            _ => {}
        };
    }
    fn on_connect(&self, _: SocketAddr, send_channel: SendChannel) -> Option<Self::Id> {
        let id = Id::new_v4();
        self.to_spawn.write().unwrap().insert(id, send_channel);
        Some(id)
    }
    fn on_disconnect(&self, id: Self::Id) {
        self.to_despawn.write().unwrap().insert(id);
    }
}
