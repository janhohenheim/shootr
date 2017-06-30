pub mod comp;
pub mod res;
pub mod sys;

extern crate specs;
extern crate chrono;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;



pub fn main_loop() {
    let mut world = World::new();
    world.register::<comp::Pos>();
    world.register::<comp::Vel>();

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
    let mut renderer = DispatcherBuilder::new()
        .add(sys::Render, "render", &[])
        .build();
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
        world.add_resource(res::TimeProgess(progress));
        renderer.dispatch(&mut world.res);
    }
}

fn elapsed_time(from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>) -> i64 {
    to.signed_duration_since(from).num_microseconds().expect(
        "Too much time passed between DateTimes",
    )
}
