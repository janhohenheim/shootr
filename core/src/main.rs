extern crate shootr;

extern crate specs;
extern crate chrono;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;

use shootr::engine::{Msg, Engine, EventHandler, Id};
use shootr::ecs::{comp, sys, res};
use res::Ids;
use std::sync::{Arc, RwLock};

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
                .with(comp::Vel { x: 3, y: 2 })
                .with(comp::Pos { x: 500, y: 500 })
                .build();
        }


        let mut physics = DispatcherBuilder::new()
            .add(sys::Physics, "physics", &[])
            .build();
        let mut renderer = DispatcherBuilder::new().add(sys::Send, "send", &[]).build();
        physics.dispatch(&mut world.res);

        let mut lag: i64 = 0;
        let mut previous = Utc::now();
        const MS_PER_UPDATE: i64 = 6000;

        loop {
            let current = Utc::now();
            let elapsed = elapsed_time(previous, current);
            previous = current;
            lag += elapsed;
            while lag >= MS_PER_UPDATE {
                physics.dispatch(&mut world.res);
                lag -= MS_PER_UPDATE;
            }
            let progress = lag as f64 / MS_PER_UPDATE as f64;
            world.add_resource(res::TimeProgress(progress));
            renderer.dispatch(&mut world.res);
        }
    }
    fn message(&self, msg: &Msg) {
        println!("Received message: {}", msg.content);
    }
    fn connect(&self, id: Id) -> bool {
        self.ids.write().unwrap().push(id);
        true
    }
    fn disconnect(&self, id: Id) {}
}



fn elapsed_time(from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>) -> i64 {
    to.signed_duration_since(from).num_microseconds().expect(
        "Too much time passed between DateTimes",
    )
}
