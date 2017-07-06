extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate dotenv;
extern crate native_tls;

use self::dotenv::dotenv;

use self::websocket::message::OwnedMessage;
use self::websocket::server::{WsServer, InvalidConnection};
use self::websocket::async::Server;

use self::tokio_core::reactor::{Handle, Remote, Core};
use self::tokio_core::net::TcpListener;

use self::futures::{Future, Sink, Stream};
use self::futures::sync::mpsc;
use self::futures_cpupool::CpuPool;

use self::native_tls::{Pkcs12, TlsAcceptor};

use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::fmt::Debug;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

use model::ClientState;
use util::read_env_var;

pub type Id = u32;

type SinkContent = websocket::client::async::Framed<
    websocket::client::async::TlsStream<tokio_core::net::TcpStream>,
    websocket::async::MessageCodec<OwnedMessage>,
>;
type SplitSink = futures::stream::SplitSink<SinkContent>;

#[derive(Clone)]
pub struct Engine {
    pub send_channel: mpsc::UnboundedSender<(Id, Arc<RwLock<ClientState>>)>,
    pub remote: Remote,
}

#[derive(Serialize, Debug)]
pub struct Msg {
    pub id: Id,
    pub content: String,
}

pub trait EventHandler {
    fn new(engine: Engine) -> Self;
    fn main_loop(&self);
    fn message(&self, msg: &Msg);
    fn connect(&self, id: Id) -> bool;
    fn disconnect(&self, id: Id);
}

pub fn execute<T>()
where
    T: EventHandler + Send + Sync + 'static,
{
    dotenv().ok();

    let mut core = Core::new().expect("Failed to create Tokio event loop");
    let handle = core.handle();
    let remote = core.remote();

    let server = build_server(&handle);
    let pool = Arc::new(RwLock::new(CpuPool::new_num_cpus()));
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
    let (send_channel_out, send_channel_in) = mpsc::unbounded();

    let engine = Engine {
        send_channel: send_channel_out.clone(),
        remote: remote.clone(),
    };
    let event_handler = T::new(engine);
    let event_handler = Arc::new(event_handler);

    let conn_id = Rc::new(RefCell::new(Counter::new()));
    let connections_inner = connections.clone();
    let event_handler_inner = event_handler.clone();
    // Handle new connection
    let connection_handler = server
        .incoming()
        .map(|(upgrade, addr)| Some((upgrade, addr)))
        .or_else(|InvalidConnection { error, .. }| {
            println!("Connection dropped: {}", error);
            Ok(None)
        })
        .for_each(move |conn| {
            if conn.is_none() {
                return Ok(());
            }
            let (upgrade, addr) = conn.unwrap();
            let event_handler = event_handler_inner.clone();
            let connections_inner = connections_inner.clone();
            println!("Got a connection from: {}", addr);
            let channel = receive_channel_out.clone();
            let conn_id = conn_id.clone();
            let f = upgrade.accept().and_then(move |(framed, _)| {
                let id = conn_id.borrow_mut().next().expect(
                    "maximum amount of ids reached",
                );
                if !event_handler.connect(id) {
                    return Ok(());
                }
                let (sink, stream) = framed.split();
                channel.send((id, stream)).wait().unwrap();
                connections_inner.write().unwrap().insert(id, sink);
                Ok(())
            });
            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|e: ()| e);


    // Handle receiving messages from a client
    let remote_inner = remote.clone();
    let connections_inner = connections.clone();
    let event_handler_inner = event_handler.clone();
    let receive_handler = pool.read()
        .unwrap()
        .spawn_fn(|| {
            receive_channel_in.for_each(move |(id, stream)| {
                let connections = connections_inner.clone();
                let event_handler = event_handler_inner.clone();
                remote_inner.spawn(move |_| {
                    let connections = connections.clone();
                    let event_handler = event_handler.clone();
                    stream
                        .for_each(move |msg| {
                            process_message(id, &msg, &*event_handler, connections.clone());
                            Ok(())
                        })
                        .map_err(|e| println!("Error while receiving messages: {}", e))
                });
                Ok(())
            })
        })
        .map_err(|e| println!("Error while receiving messages: {:?}", e));


    // Handle sending messages to a client
    let connections_inner = connections.clone();
    let send_handler = pool.read()
        .unwrap()
        .spawn_fn(move || {
            let connections = connections_inner.clone();
            send_channel_in.for_each(move |(id, state): (Id, Arc<RwLock<ClientState>>)| {
                let mut sink_guard = connections.write().unwrap();
                // Todo: don't even send invalid ids
                if let Some(mut sink) = sink_guard.get_mut(&id) {
                    let msg = serde_json::to_string(state.read().unwrap().deref()).unwrap();
                    //println!("Sending message: {}", msg);
                    sink.start_send(OwnedMessage::Text(msg)).expect(
                        "Failed to start sending message",
                    );
                    sink.poll_complete().expect("Failed to send message");
                }
                Ok(())
            })
        })
        .map_err(|e| println!("Error while sending messages: {:?}", e));

    // Run maIn loop
    let main_fn = pool.read()
        .unwrap()
        .spawn_fn(move || {
            event_handler.main_loop();
            Ok::<(), ()>(())
        })
        .map_err(|e| println!("Error in main callback function: {:?}", e));
    let handlers = main_fn.select2(connection_handler.select2(
        receive_handler.select(send_handler),
    ));
    core.run(handlers)
        .map_err(|_| println!("Unspecified error while running core loop"))
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

fn build_server(handle: &Handle) -> WsServer<TlsAcceptor, TcpListener> {
    let mut file = File::open(&read_env_var("CERT_FILE")).expect("Failed to open certificate file");
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Pkcs12::from_der(&pkcs12, &read_env_var("CERT_PW")).expect(
        "Failed to open certificate file with given password",
    );

    let address = format!("localhost:{}", read_env_var("CORE_PORT"));
    let acceptor = TlsAcceptor::builder(pkcs12).unwrap().build().expect(
        "Failed to build TLS acceptor",
    );
    Server::bind_secure(address, acceptor, &handle).expect("Failed to create server")
}

fn process_message<T>(
    id: Id,
    msg: &OwnedMessage,
    event_handler: &T,
    connections: Arc<RwLock<HashMap<Id, SplitSink>>>,
) where
    T: EventHandler,
{
    match *msg {
        OwnedMessage::Text(ref content) => {
            let msg = Msg {
                id,
                content: content.clone(),
            };
            event_handler.message(&msg);
        }
        OwnedMessage::Close(_) => {
            connections
                .write()
                .unwrap()
                .remove(&id)
                .and_then(|_| Some(println!("Client with id {} disconnected", id)))
                .expect("Tried to remove id that was not in list");
            event_handler.disconnect(id);
        }
        _ => {}
    };
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
