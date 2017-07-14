extern crate shootr;

extern crate specs;
extern crate chrono;
extern crate serde_json;
#[macro_use]
extern crate maplit;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;

use shootr::engine::{Msg, Engine, EventHandler, Id};
use shootr::util::{read_env_var, elapsed_ms};
use shootr::model::{Bounds, Acc, Vel, Pos, InputMsg, PlayerInput, KeyState, Bounciness};
use shootr::system::{Physics, Sending, InputHandler, Bounce};

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
    inputs: Arc<RwLock<HashMap<Id, PlayerInput>>>,
}

impl Handler {
    fn prepare_world(&self, world: &mut World) {
        world.register::<Pos>();
        world.register::<Vel>();
        world.register::<Acc>();
        world.register::<PlayerInput>();
        world.register::<Bounciness>();

        world.add_resource(self.engine.clone());
        world.add_resource(self.ids.clone());
        world.add_resource(self.inputs.clone());
        world.add_resource(Bounds {
            min: Acc {
                x: -5,
                y: -5,
            },
            max: Acc {
                x: 5,
                y: 5,
            },
        });
        world.add_resource(Bounds {
            min: Vel {
                x: -50,
                y: -50,
            },
            max: Vel {
                x: 50,
                y: 50,
            },
        });
        world.add_resource(Bounds {
            min: Pos {
                x: 0,
                y: 0,
            },
            max: Pos {
                x: 1000,
                y: 1000,
            },
        });


        // Ball
        world
            .create_entity()
            .with(Vel { x: 15, y: 10 })
            .with(Pos { x: 500, y: 500 })
            .with(Bounciness {})
            .build();

        // Player
        world
            .create_entity()
            .with(Acc { x: 0, y: 0 })
            .with(Vel { x: 0, y: 0 })
            .with(Pos { x: 10, y: 500 })
            .with(PlayerInput { key_states: HashMap::new() })
            .build();
    }
}

impl EventHandler for Handler {
    fn new(engine: Engine) -> Self {
        Handler {
            engine,
            ids: Arc::new(RwLock::new(Vec::new())),
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
        if let Ok(input) = serde_json::from_str::<InputMsg>(&msg.content) {
            println!(
                "[{}] Client #{}:\tkey {:?} is pressed:\t{}",
                input.id,
                msg.id,
                input.key,
                input.pressed
            );
            let mut inputs = self.inputs.write().unwrap();
            let mut key_state = KeyState {
                pressed: input.pressed,
                fired: false,
            };

            if inputs.contains_key(&msg.id) {
                let key_states = &mut inputs.get_mut(&msg.id).unwrap().key_states;
                if let Some(last) = key_states.get_mut(&input.key) {
                    key_state.fired = last.pressed && !key_state.pressed;
                }
                key_states.insert(input.key, key_state);
            } else {
                let key_states = PlayerInput { key_states: hashmap![input.key => key_state] };
                inputs.insert(msg.id, key_states);
            }
        } else {
            println!("Client #{}:\tinvalid message ({})", msg.id, msg.content);
        }
    }
    fn connect(&self, id: Id) -> bool {
        self.ids.write().unwrap().push(id);
        true
    }
    fn disconnect(&self, id: Id) {
        let mut ids = self.ids.write().unwrap();
        let pos = ids.iter()
            .position(|&x| x == id)
            .expect("Tried to remove id that was not added in the first place");

        ids.remove(pos);
    }
}
