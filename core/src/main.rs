extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate shootr;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::{Handle, Remote, Core};

use futures::{Future, BoxFuture, Sink, Stream};
use futures::future::{self, Loop};
use futures::sync::mpsc;
use futures_cpupool::{CpuPool, CpuFuture};

use std::sync::{RwLock, Arc};
use std::thread;
use std::rc::Rc;
use std::fmt::Debug;
use std::time::Duration;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

use shootr::model::ClientState;
use shootr::ecs::res;
use shootr::engine;

fn main() {
    shootr::engine::execute(main_loop);
}



fn main_loop(engine: engine::Engine) -> BoxFuture<(), ()> {
    future::loop_fn(engine, |conn_info| {
        thread::sleep(Duration::from_millis(100));

        let should_continue = update(
            conn_info.connections.clone(),
            conn_info.channel.clone(),
            &conn_info.remote,
        );
        match should_continue {
            Ok(true) => Ok(Loop::Continue(conn_info)),
            Ok(false) => Ok(Loop::Break(())),
            Err(()) => Err(()),
        }
    }).boxed()
}

type Id = u32;

type SinkContent = websocket::client::async::Framed<
    tokio_core::net::TcpStream,
    websocket::async::MessageCodec<OwnedMessage>,
>;
type SplitSink = futures::stream::SplitSink<SinkContent>;
// Represents one tick in the main loop
fn update(
    connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
    channel: mpsc::UnboundedSender<(Id, Arc<RwLock<ClientState>>)>,
    remote: &Remote,
) -> Result<bool, ()> {
    remote.spawn(move |handle| {
        let state = Arc::new(RwLock::new(ClientState {}));
        for (id, _) in connections.read().unwrap().iter() {
            let f = channel.clone().send((*id, state.clone()));
            shootr::engine::spawn_future(f, "Send message to write handler", handle);
        }
        Ok(())
    });
    Ok(true)
}
