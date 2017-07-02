extern crate shootr;

extern crate specs;
extern crate chrono;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;

use shootr::engine::{Msg, Engine};
use shootr::ecs::{comp, sys, res};

fn main() {
    shootr::engine::execute(main_loop, handle_message);
}



fn main_loop(engine: Engine) {
    let mut world = World::new();
    world.register::<comp::Pos>();
    world.register::<comp::Vel>();
    world.add_resource(engine);
    for _ in 0..1 {
        world
            .create_entity()
            .with(comp::Vel { x: 1, y: 1 })
            .with(comp::Pos { x: 0, y: 0 })
            .build();
    }


    let mut physics = DispatcherBuilder::new()
        .add(sys::Physics, "physics", &[])
        .build();
    let mut renderer = DispatcherBuilder::new().add(sys::Send, "send", &[]).build();
    physics.dispatch(&mut world.res);

    let mut lag: i64 = 0;
    let mut previous = Utc::now();
    const MS_PER_UPDATE: i64 = 150;

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

fn handle_message(_: &Engine, msg: &Msg) {
    println!("Received message: {}", msg.content);
}


fn elapsed_time(from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>) -> i64 {
    to.signed_duration_since(from).num_microseconds().expect(
        "Too much time passed between DateTimes",
    )
}
