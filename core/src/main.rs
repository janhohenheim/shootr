extern crate shootr;

extern crate specs;
extern crate chrono;
extern crate serde_json;

use self::specs::{DispatcherBuilder, World, Entity};
use self::chrono::prelude::*;

use shootr::engine::{EventHandler, OwnedMessage, SendChannel};
use shootr::util::{read_env_var, elapsed_ms};
use shootr::model::comp::{Vel, Pos, Bounciness, Connect, Disconnect, Player, Ping};
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
    shootr::engine::execute::<Handler>();
}

struct Handler {
    id_entity: RwLock<HashMap<Id, Entity>>,
    to_spawn: RwLock<HashMap<Id, SendChannel>>,
    to_despawn: RwLock<HashSet<Id>>,
    inputs: Arc<RwLock<HashMap<Id, Vec<KeyState>>>>,
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
            let entity = id_entity.remove(&id).expect(
                "Tried to remove id that was not there",
            );
            world.write::<Disconnect>().insert(entity, Disconnect {});
        }
    }

    fn register_pings(&self, world: &mut World) {
        for (_, entity) in self.id_entity.read().unwrap().iter() {
            world.write::<Ping>().insert(entity.clone(), Ping {});
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

        let mut lag: u64 = 0;
        let mut previous = Utc::now();
        let updates_per_sec = read_env_var("CORE_UPDATES_PER_SEC").parse::<u64>().expect(
            "Failed to parse environmental variable as integer",
        );
        let ms_per_update = 1000 / updates_per_sec;
        let mut ping_timer = 0;
        let ping_interval = 500;
        loop {
            let current = Utc::now();
            let elapsed = elapsed_ms(previous, current).expect("Time went backwards");
            previous = current;
            lag += elapsed;
            ping_timer += elapsed;

            self.register_spawns(&mut world);
            if ping_timer > ping_interval {
                self.register_pings(&mut world);
                ping_timer = 0;
            }

            while lag >= ms_per_update {
                updater.dispatch(&mut world.res);
                lag -= ms_per_update;
            }
            sender.dispatch(&mut world.res);
            world.maintain();

            sleep(Duration::from_millis(ms_per_update - lag));
        }
    }

    fn on_message(&self, id: Self::Id, msg: OwnedMessage) {
        match msg {
            OwnedMessage::Text(ref txt) => self.handle_text(id, txt),
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
