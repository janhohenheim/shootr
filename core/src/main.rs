#![feature(conservative_impl_trait)]

extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;


use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use tokio_core::reactor::{Handle, Remote, Core};

use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};
use futures::sync::mpsc;
use futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::thread;
use std::rc::Rc;
use std::fmt::Debug;
use std::time::Duration;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

type Id = u32;

fn main() {
    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let server = Server::bind("localhost:8081", &handle).expect("Failed to create server");
    let pool = Rc::new(CpuPool::new_num_cpus());
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let state = Arc::new(RwLock::new(State::new()));
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();

    let conn_id = Rc::new(RefCell::new(Counter::new()));
    let connections_inner = connections.clone();
    // Handle new connection
    let connection_handler = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            let connections_inner = connections_inner.clone();
            println!("Got a connection from: {}", addr);
            let channel = receive_channel_out.clone();
            let handle_inner = handle.clone();
            let conn_id = conn_id.clone();
            let f = upgrade
                .use_protocol("rust-websocket")
                .accept()
                .and_then(move |(framed, _)| {
                    let id = conn_id
                        .borrow_mut()
                        .next()
                        .expect("maximum amount of ids reached");
                    let (sink, stream) = framed.split();
                    let f = channel.send((id, stream));
                    spawn_future(f, "Senk stream to connection pool", &handle_inner);
                    connections_inner.write().unwrap().insert(id, sink);
                    Ok(())
                });
            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|_| ());


    // Handle receiving messages from a client
    let state_read = state.clone();
    let remote = core.remote();
    let receive_handler = pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let state_read = state_read.clone();
            remote.spawn(move |_| {
                stream.for_each(move |msg| {
                                    process_message(id, &msg, &mut state_read.write().unwrap());
                                    Ok(())
                                })
                    .map_err(|_| ())
            });
            Ok(())
        })
    });

    let (send_channel_out, send_channel_in) = mpsc::unbounded();

    // Handle sending messages to a client
    let connections_inner = connections.clone();
    let send_handler = pool.spawn_fn(move || {
        let connections = connections_inner.clone();
        send_channel_in.for_each(move |(id, state): (Id, Arc<RwLock<State>>)| {
                let connections = connections.clone();
                let mut connections = connections.write().unwrap();

                let sink = connections.remove(&id).expect("Tried to send to invalid client id");

                let msg = serde_json::to_string(state.read().unwrap().deref()).unwrap();
                println!("Sending message: {}", msg);
                let sink = sink.send(OwnedMessage::Text(msg))
                    .wait()
                    .expect("Error while sending to client");

                connections.insert(id, sink);
                Ok(())
            })
            .map_err(|_| ())
    });

    // Main 'logic' loop
    let remote = core.remote();
    let game_loop = pool.spawn_fn(move || {
        future::loop_fn(send_channel_out, move |send_channel_out| {
            thread::sleep(Duration::from_millis(100));

            let should_continue = update(connections.clone(),
                                          state.clone(),
                                          send_channel_out.clone(),
                                          &remote);
            match should_continue {
                Ok(true) => Ok(Loop::Continue(send_channel_out)),
                Ok(false) => Ok(Loop::Break(())),
                Err(()) => Err(()),
            }
        })
    });

    let handlers =
        game_loop.select2(connection_handler.select2(receive_handler.select(send_handler)));
    core.run(handlers).map_err(|_| println!("Error while running core loop")).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static,
          E: Debug
{
    handle.spawn(f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
                     .map(move |_| println!("{}: Finished.", desc)));
}


fn process_message(_: u32, msg: &OwnedMessage, _: &mut State) {
    if let OwnedMessage::Text(ref txt) = *msg {
        println!("Received message: {}", txt);
        //state.msg = txt.clone();
    }
}

type SinkContent =
    websocket::client::async::Framed<tokio_core::net::TcpStream,
                                     websocket::async::MessageCodec<websocket::OwnedMessage>>;
type SplitSink = futures::stream::SplitSink<SinkContent>;
// Represents one tick in the main loop
fn update(connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
           state: Arc<RwLock<State>>,
           channel: futures::sync::mpsc::UnboundedSender<(Id, Arc<RwLock<State>>)>,
           remote: &Remote)
           -> Result<bool, ()> {
    remote.spawn(move |handle| {
        for (id, _) in connections.read().unwrap().iter() {
            let f = channel.clone().send((*id, state.clone()));
            spawn_future(f, "Send message to write handler", handle);
        }
        Ok(())
    });
    Ok(true)
}

struct Counter {
    count: Id,
}
impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }
}


impl Iterator for Counter {
    type Item = Id;

    fn next(&mut self) -> Option<Id> {
        if self.count != Id::max_value() {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}


#[derive(Serialize, Deserialize)]
struct State {
    positions: Vec<Pos>,
}

impl State {
    fn new() -> Self {
        Self { positions: Vec::new() }
    }
}


#[derive(Serialize, Deserialize)]
struct Pos {
    x: i32,
    y: i32,
}

/*
impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn add_x(&mut self, x: i32) -> &mut Self {
        self.x += x;
        self
    }


    fn add_y(&mut self, y: i32) -> &mut Self {
        self.y += y;
        self
    }

    fn add(&mut self, x: i32, y: i32) -> &mut Self {
        self.x += x;
        self.y += y;
        self
    }
}
*/
