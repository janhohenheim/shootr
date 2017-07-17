extern crate shootr;

extern crate specs;
extern crate chrono;
extern crate serde_json;

use self::specs::{DispatcherBuilder, World, Entity};
use self::chrono::prelude::*;

use shootr::engine::{Msg, Engine, EventHandler, Id};
use shootr::util::{read_env_var, elapsed_ms};
use shootr::model::comp::{Acc, Vel, Pos, Bounciness, PlayerId, Friction};
use shootr::model::client::InputMsg;
use shootr::model::game::{KeyState, PlayerInputMap, PlayerInput, Spawnable};
use shootr::system::{Physics, Sending, InputHandler, Bounce};
use shootr::bootstrap;

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;


fn main() {
    shootr::engine::execute::<Handler>();
}

struct Handler {
    engine: Engine,
    ids: Arc<RwLock<Vec<Id>>>,
    inputs: PlayerInputMap,
    id_entities: RwLock<HashMap<Id, Entity>>,
    spawn_list: RwLock<Vec<Spawnable>>,
    despawn_list: RwLock<Vec<Entity>>
}

impl Handler {
    fn prepare_world(&self, world: &mut World) {
        bootstrap::prepare_world(world);

        world.add_resource(self.engine.clone());
        world.add_resource(self.ids.clone());
        world.add_resource(self.inputs.clone());

        // Ball
        world
            .create_entity()
            .with(Vel { x: 15, y: 10 })
            .with(Pos { x: 500, y: 500 })
            .with(Bounciness {})
            .build();
    }

    fn despawn(&self, world: &mut World) {
        let mut despawn_list = self.despawn_list.write().unwrap();
        for to_despawn in despawn_list.drain(..) {
            world.delete_entity(to_despawn);
        };
    }

    fn spawn(&self, world: &mut World) {
        let mut spawn_list = self.spawn_list.write().unwrap();
        for to_spawn in spawn_list.drain(..) {
            match to_spawn {
                Spawnable::Player(id) => {
                    let x = if id % 2 == 0 { 20 } else { 980 };
                    let entity = world
                        .create_entity()
                        .with(Acc { x: 0, y: 0 })
                        .with(Vel { x: 0, y: 0 })
                        .with(Pos { x, y: 500 })
                        .with(Friction(2))
                        .with(PlayerId(id))
                        .build();
                    self.id_entities.write().unwrap().insert(id, entity);
                }
            };
        };
    }
}

impl EventHandler for Handler {
    fn new(engine: Engine) -> Self {
        Handler {
            engine,
            ids: Arc::new(RwLock::new(Vec::new())),
            inputs: Arc::new(RwLock::new(HashMap::new())),
            id_entities: RwLock::new(HashMap::new()),
            spawn_list: RwLock::new(Vec::new()),
            despawn_list: RwLock::new(Vec::new()),
        }
    }
    fn main_loop(&self) {
        let mut world = World::new();
        self.prepare_world(&mut world);

        let mut updater = DispatcherBuilder::new()
            .add(InputHandler, "input_handler", &[])
            .add(Physics, "physics", &["input_handler"])
            .add(Bounce, "bounce", &["physics"])
            .build();
        let mut sender = DispatcherBuilder::new()
            .add(Sending, "sending", &[])
            .build();

        let mut lag: u64 = 0;
        let mut previous = Utc::now();
        let updates_per_sec = read_env_var("CORE_UPDATES_PER_SEC")
            .parse::<u64>()
            .expect("Failed to parse environmental variable as integer");
        let ms_per_update = 1000 / updates_per_sec;
        loop {
            let current = Utc::now();
            let elapsed = elapsed_ms(previous, current);
            previous = current;
            lag += elapsed;
            self.despawn(&mut world);
            self.spawn(&mut world);

            while lag >= ms_per_update {
                updater.dispatch(&mut world.res);
                lag -= ms_per_update;
            }
            sender.dispatch(&mut world.res);
            world.maintain();
            sleep(Duration::from_millis(ms_per_update - lag));
        }
    }

    fn message(&self, msg: &Msg) {
        let input = serde_json::from_str::<InputMsg>(&msg.content);
        if input.is_err() {
            println!("Client #{}:\tinvalid message ({})", msg.id, msg.content);
            return;
        }
        let input = input.unwrap();

        let mut key_state = KeyState {
            pressed: input.pressed,
            fired: false,
        };

        let inputs = self.inputs.read().unwrap();
        // guaranteed to contain key as connect() had to be called before
        let key_states = &mut inputs.get(&msg.id).unwrap().write().unwrap().key_states;

        if let Some(last) = key_states.get_mut(&input.key) {
            key_state.fired = last.pressed && !key_state.pressed;
        }
        key_states.insert(input.key, key_state);
    }
    fn connect(&self, id: Id) -> bool {
        self.ids.write().unwrap().push(id);
        let id_state = RwLock::new(PlayerInput {key_states: HashMap::new()});
        self.spawn_list.write().unwrap().push(Spawnable::Player(id));
        self.inputs.write().unwrap().insert(id, id_state);
        true
    }
    fn disconnect(&self, id: Id) {
        let mut ids = self.ids.write().unwrap();
        let pos = ids.iter()
            .position(|&x| x == id)
            .expect("Tried to remove id that was not added in the first place");
        let id = ids.remove(pos);
        if let Some(entity) = self.id_entities.write().unwrap().remove(&id) {
            self.despawn_list.write().unwrap().push(entity);
        }
    }
}
