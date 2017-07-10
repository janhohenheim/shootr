extern crate shootr;

extern crate specs;
extern crate chrono;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;

use shootr::engine::{Msg, Engine, EventHandler, Id};
use shootr::ecs::{comp, sys, res};
use shootr::util::{read_env_var, elapsed_time};
use res::Ids;

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    shootr::engine::execute::<Handler>();
}
struct Handler {
    engine: Engine,
    ids: Ids,
}
impl EventHandler for Handler {
    fn new(engine: Engine) -> Self {
        Handler {
            engine,
            ids: Ids(Arc::new(RwLock::new(Vec::new()))),
        }
    }
    fn main_loop(&self) {
        let mut world = World::new();
        world.register::<comp::Pos>();
        world.register::<comp::Vel>();
        world.add_resource(self.engine.clone());
        world.add_resource(self.ids.clone());
        for _ in 0..1 {
            world
                .create_entity()
                .with(comp::Vel { x: 8, y: 6 })
                .with(comp::Pos { x: 500, y: 500 })
                .build();
        }


        let mut physics = DispatcherBuilder::new()
            .add(sys::Physics, "physics", &[])
            .build();
        let mut renderer = DispatcherBuilder::new().add(sys::Send, "send", &[]).build();
        physics.dispatch(&mut world.res);

        let mut lag: u64 = 0;
        let mut previous = Utc::now();
        let updates_per_sec = read_env_var("CORE_UPDATES_PER_SEC").parse::<u64>().expect(
            "Failed to parse environmental variable as integer",
        );
        let ms_per_update = 1000 / updates_per_sec;
        loop {
            let current = Utc::now();
            let elapsed = elapsed_time(previous, current);
            previous = current;
            lag += elapsed;
            while lag >= ms_per_update {
                physics.dispatch(&mut world.res);
                lag -= ms_per_update;
            }
            renderer.dispatch(&mut world.res);
            sleep(Duration::from_millis(ms_per_update - lag));
        }
    }
    fn message(&self, msg: &Msg) {
        println!("Received message: {}", msg.content);
    }
    fn connect(&self, id: Id) -> bool {
        self.ids.write().unwrap().push(id);
        true
    }
    fn disconnect(&self, id: Id) {
        let mut ids = self.ids.write().unwrap();
        let pos = ids.iter().position(|&x| x == id).expect(
            "Tried to remove id that was not added in the first place",
        );

        ids.remove(pos);
    }
}
