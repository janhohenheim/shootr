extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;


use self::websocket::message::OwnedMessage;
use self::websocket::server::InvalidConnection;
use self::websocket::async::Server;

use self::tokio_core::reactor::{Handle, Remote, Core};

use self::futures::{Future, BoxFuture, Sink, Stream, IntoFuture};
use self::futures::future::{self, Loop};
use self::futures::sync::mpsc;
use self::futures_cpupool::{CpuPool, CpuFuture};

use std::sync::{RwLock, Arc};
use std::thread;
use std::rc::Rc;
use std::fmt::Debug;
use std::time::Duration;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

use model::ClientState;

type Id = u32;

pub fn execute<F>(function: F)
where
    F: FnOnce(Engine) -> BoxFuture<(), ()> + Send + 'static,
{
    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("localhost:8081", &handle).expect("Failed to create server");
    let pool = Arc::new(RwLock::new(CpuPool::new_num_cpus()));
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
    let remote_inner = remote.clone();
    let receive_handler = pool.read().unwrap().spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let state_read = state_read.clone();
            remote_inner.spawn(move |_| {
                stream
                    .for_each(move |msg| {
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
    let remote_inner = remote.clone();
    let send_handler = pool.read().unwrap().spawn_fn(move || {
        let connections = connections_inner.clone();
        let remote = remote_inner.clone();
        send_channel_in
            .for_each(move |(id, state): (Id, Arc<RwLock<ClientState>>)| {
                let connections = connections.clone();
                let sink = connections.write().unwrap().remove(&id).expect(
                    "Tried to send to invalid client id",
                );

                let msg = serde_json::to_string(state.read().unwrap().deref()).unwrap();
                println!("Sending message: {}", msg);
                let f = sink.send(OwnedMessage::Text(msg)).and_then(move |sink| {
                    connections.write().unwrap().insert(id, sink);
                    Ok(())
                });
                remote.spawn(move |_| f.map_err(|_| ()));
                Ok(())
            })
            .map_err(|_| ())
    });

    let engine = Engine {
        connections: connections,
        channel: send_channel_out,
        remote: remote,
        pool: pool.clone(),
    };
    let function = pool.read().unwrap().spawn_fn(move || function(engine));
    let handlers = function.select2(connection_handler.select2(
        receive_handler.select(send_handler),
    ));
    core.run(handlers)
        .map_err(|_| println!("Error while running core loop"))
        .unwrap();
}

pub fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
where
    F: Future<Item = I, Error = E> + 'static,
    E: Debug,
{
    handle.spawn(
        f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
            .map(move |_| println!("{}: Finished.", desc)),
    );
}


fn process_message(_: u32, msg: &OwnedMessage, _: &mut State) {
    if let OwnedMessage::Text(ref txt) = *msg {
        println!("Received message: {}", txt);
        //state.msg = txt.clone();
    }
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



type SinkContent = websocket::client::async::Framed<
    tokio_core::net::TcpStream,
    websocket::async::MessageCodec<OwnedMessage>,
>;
type SplitSink = futures::stream::SplitSink<SinkContent>;
#[derive(Clone)]
pub struct Engine {
    pub connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
    pub channel: mpsc::UnboundedSender<(Id, Arc<RwLock<ClientState>>)>,
    pub remote: Remote,
    pub pool: Arc<RwLock<CpuPool>>,
}
