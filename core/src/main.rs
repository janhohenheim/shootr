extern crate shootr;

extern crate specs;
extern crate chrono;
extern crate serde_json;

use self::specs::{DispatcherBuilder, World, Entity};
use self::chrono::prelude::*;

use shootr::engine::{EventHandler, OwnedMessage, SendChannel};
use shootr::util::{read_env_var, elapsed_ms};
use shootr::model::comp::{Acc, Vel, Pos, Bounciness, Friction, Connect, Disconnect, Player};
use shootr::model::client::InputMsg;
use shootr::model::game::{KeyState, PlayerInputMap, PlayerInput, Vector};
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
    world: Arc<RwLock<World>>,
    inputs: PlayerInputMap,
}

impl Handler {
    fn prepare_world(&self) {
        let mut world = self.world.write().unwrap();
        bootstrap::prepare_world(&mut *world);
        world.add_resource(self.inputs.clone());

        // Ball
        world
            .create_entity()
            .with(Vel::from(Vector { x: 15, y: 10 }))
            .with(Pos::from(Vector { x: 500, y: 500 }))
            .with(Bounciness {})
            .build();
    }


    fn handle_text(&self, id: <Handler as EventHandler>::Id, msg: &str) {
        let input = serde_json::from_str::<InputMsg>(&msg);
        if input.is_err() {
            println!("invalid message ({})", msg);
            return;
        }
        let input = input.unwrap();

        let mut key_state = KeyState {
            pressed: input.pressed,
            fired: false,
        };

        let inputs = self.inputs.read().unwrap();
        // guaranteed to contain key as connect() had to be called before
        let key_states = &mut inputs.get(&id).unwrap().write().unwrap().key_states;

        if let Some(last) = key_states.get_mut(&input.key) {
            key_state.fired = last.pressed && !key_state.pressed;
        }
        key_states.insert(input.key, key_state);
    }
}

impl EventHandler for Handler {
    type Id = Entity;

    fn new() -> Self {
        Handler {
            world: Arc::new(RwLock::new(World::new())),
            inputs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn main_loop(&self) {
        self.prepare_world();

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
        let updates_per_sec = read_env_var("CORE_UPDATES_PER_SEC").parse::<u64>().expect(
            "Failed to parse environmental variable as integer",
        );
        let ms_per_update = 1000 / updates_per_sec;
        loop {
            let current = Utc::now();
            let elapsed = elapsed_ms(previous, current).expect("Time went backwards");
            previous = current;
            lag += elapsed;
            {
                let mut world = self.world.write().unwrap();
                while lag >= ms_per_update {
                    updater.dispatch(&mut world.res);
                    lag -= ms_per_update;
                }
                sender.dispatch(&mut world.res);
                world.maintain();
            }
            sleep(Duration::from_millis(ms_per_update - lag));
        }
    }

    fn message(&self, id: Self::Id, msg: OwnedMessage) {
        match msg {
            OwnedMessage::Text(ref txt) => self.handle_text(id, txt),
            _ => {}
        };
    }
    fn connect(&self, send_channel: SendChannel) -> Option<Self::Id> {
        let world = self.world.write().unwrap();
        let entity = world
            .create_entity()
            .with(Connect {})
            .with(Player { send_channel })
            .build();

        let id_state = RwLock::new(PlayerInput { key_states: HashMap::new() });
        self.inputs.write().unwrap().insert(
            entity.clone(),
            id_state,
        );
        Some(entity)
    }
    fn disconnect(&self, id: Self::Id) {
        let world = self.world.write().unwrap();
        world.write::<Disconnect>().insert(id, Disconnect {});
    }
}
