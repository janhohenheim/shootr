extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;

extern crate shootr;

extern crate specs;
extern crate chrono;

use self::specs::{DispatcherBuilder, World};
use self::chrono::prelude::*;


use websocket::message::OwnedMessage;

use tokio_core::reactor::Remote;

use futures::{Future, BoxFuture, Sink};
use futures::future::{self, Loop};
use futures::sync::mpsc;

use std::sync::{RwLock, Arc};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use shootr::model::ClientState;
use shootr::engine;
use shootr::ecs::{comp, sys, res};

fn main() {
    shootr::engine::execute(main_loop);
}



fn main_loop(engine: engine::Engine) {
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
        world.add_resource(res::TimeProgess(progress));
        renderer.dispatch(&mut world.res);
    }
}


fn elapsed_time(from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>) -> i64 {
    to.signed_duration_since(from).num_microseconds().expect(
        "Too much time passed between DateTimes",
    )
}
