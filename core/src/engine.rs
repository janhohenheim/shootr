extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;


use self::websocket::message::OwnedMessage;
use self::websocket::server::InvalidConnection;
use self::websocket::async::Server;

use self::tokio_core::reactor::{Handle, Remote, Core};

use self::futures::{Future, Sink, Stream};
use self::futures::sync::mpsc;
use self::futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::fmt::Debug;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

use model::ClientState;

pub type Id = u32;

type SinkContent = websocket::client::async::Framed<
    tokio_core::net::TcpStream,
    websocket::async::MessageCodec<OwnedMessage>,
>;
type SplitSink = futures::stream::SplitSink<SinkContent>;

#[derive(Clone)]
pub struct Engine {
    pub connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
    pub send_channel: mpsc::UnboundedSender<(Id, Arc<RwLock<ClientState>>)>,
    pub remote: Remote,
    pub pool: Arc<RwLock<CpuPool>>,
}


#[derive(Serialize, Debug)]
pub struct Msg {
    pub id: Id,
    pub content: String,
}

pub fn execute<Fm, Fi>(main_cb: Fm, message_cb: Fi)
where
    Fm: FnOnce(&Engine) + Send + 'static,
    Fi: Fn(&Engine, &Msg) + Sync + Send + 'static,
{
    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("localhost:8081", &handle).expect("Failed to create server");
    let pool = Arc::new(RwLock::new(CpuPool::new_num_cpus()));
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
    let (send_channel_out, send_channel_in) = mpsc::unbounded();
    
    let engine = Engine {
        connections: connections.clone(),
        send_channel: send_channel_out.clone(),
        remote: remote.clone(),
        pool: pool.clone(),
    };
    let engine = Arc::new(RwLock::new(engine));

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
                    spawn_future(f, "Send stream to connection pool", &handle_inner);
                    connections_inner.write().unwrap().insert(id, sink);
                    Ok(())
                });
            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|_| ());


    // Handle receiving messages from a client
    let remote_inner = remote.clone();
    let engine_inner = engine.clone();
    let message_cb = Arc::new(message_cb);
    let receive_handler = pool.read().unwrap().spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let engine = engine_inner.clone();
            let message_cb = message_cb.clone();
            remote_inner.spawn(move |_| {
                let engine = engine.clone();
                let message_cb = message_cb.clone();
                stream
                    .for_each(move |msg| {
                        let engine = engine.read().unwrap();
                        process_message(id, &msg, &*engine, message_cb.clone());
                        Ok(())
                    })
                    .map_err(|_| ())
            });
            Ok(())
        })
    });


    // Handle sending messages to a client
    let connections_inner = connections.clone();
    let send_handler = pool.read().unwrap().spawn_fn(move || {
        let connections = connections_inner.clone();
        send_channel_in
            .for_each(move |(id, state): (Id, Arc<RwLock<ClientState>>)| {
                let mut sink_guard = connections.write().unwrap();
                let mut sink = sink_guard.get_mut(&id).expect(
                    "Tried to send to invalid client id",
                );
                let msg = serde_json::to_string(state.read().unwrap().deref()).unwrap();
                //println!("Sending message: {}", msg);
                sink.start_send(OwnedMessage::Text(msg)).expect(
                    "Failed to start sending message",
                );
                sink.poll_complete().expect("Failed to send message");
                Ok(())
            })
            .map_err(|_| ())
    });

    let function = pool.read().unwrap().spawn_fn(move || {
        let engine = engine.read().unwrap();
        main_cb(&*engine);
        Ok::<(), ()>(())
    });
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
            .map(move |_| ()),
    );
}


fn process_message<F>(id: Id, msg: &OwnedMessage, engine: &Engine, cb: Arc<F>)
where
    F: Fn(&Engine, &Msg) + Send + 'static,
{
    if let OwnedMessage::Text(ref content) = *msg {
        let msg = Msg {
            id,
            content: content.clone(),
        };
        cb(engine, &msg);
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
