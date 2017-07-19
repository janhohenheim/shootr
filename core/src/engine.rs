extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate dotenv;

use self::dotenv::dotenv;

use self::websocket::message::OwnedMessage;
use self::websocket::server::{WsServer, InvalidConnection};
use self::websocket::async::{Server, MessageCodec};
use self::websocket::client::async::Framed;
use self::websocket::server::NoTlsAcceptor;

use self::tokio_core::reactor::{Handle, Remote, Core};
use self::tokio_core::net::{TcpStream, TcpListener};

use self::futures::{Future, Sink, Stream};
use self::futures::sync::mpsc;
use self::futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::fmt::Debug;
use std::ops::Deref;
use std::collections::HashMap;
use std::cell::RefCell;

use model::client::ClientState;
use util::read_env_var;

pub type Id = u32;

type SinkContent = Framed<TcpStream, MessageCodec<OwnedMessage>>;
type SplitSink = futures::stream::SplitSink<SinkContent>;

#[derive(Clone)]
pub struct ConnectionData {
    pub ping: u64,
    pub send_channel: mpsc::UnboundedSender<Arc<RwLock<ClientState>>>,
}
type SyncConnections = HashMap<Id, RwLock<ConnectionData>>;
type Connections = Arc<RwLock<SyncConnections>>;

#[derive(Clone)]
pub struct Engine {
    pub remote: Remote,
    pub connections: Connections,
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
    let pool = CpuPool::new(3);
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
    let (send_channel_out, send_channel_in) = mpsc::unbounded();
    
    let engine = Engine {
        connections: connections.clone(),
        remote: remote.clone(),
    };
    let event_handler = T::new(engine);
    let event_handler = Arc::new(event_handler);

    let conn_id = Rc::new(RefCell::new(IdGen::new()));
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
            let receive_channel = receive_channel_out.clone();
            let send_channel = send_channel_out.clone();
            let conn_id = conn_id.clone();
            let f = upgrade.accept().and_then(move |(framed, _)| {
                let id = conn_id.borrow_mut().next().expect(
                    "maximum amount of ids reached",
                );
                println!("Got a connection from: {}, assigned id {}", addr, id);
                if !event_handler.connect(id) {
                    return Ok(());
                }
                let (sink, stream) = framed.split();
                let (conn_out, conn_in) = mpsc::unbounded();
                send_channel.send((id, conn_in, sink)).wait().unwrap();
                receive_channel.send((id, stream)).wait().unwrap();
                let data = ConnectionData {
                    ping: 0,
                    send_channel: conn_out,
                };
                connections_inner.write().unwrap().insert(id, RwLock::new(data));
                Ok(())
            });
            spawn_future(f, "Handle new connection", &handle);
            Ok(())
        })
        .map_err(|e: ()| e);


    // Handle receiving messages from a client
    let connections_inner = connections.clone();
    let event_handler_inner = event_handler.clone();
    let remote_inner = remote.clone();
    let receive_handler = pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let connections = connections_inner.clone();
            let event_handler = event_handler_inner.clone();
            remote_inner.spawn(move |_| {
            stream
                .for_each(move |msg| {
                    process_message(id, &msg, &*event_handler, connections.clone());
                    Ok(())
                })
                .map_err(|e| println!("Error while receiving messages: {}", e))
            });
            Ok(())
        })
    }).map_err(|e| println!("Error while receiving messages: {:?}", e));


    // Handle sending messages to a client
    let event_handler_inner = event_handler.clone();
    let send_handler = pool.spawn_fn(move || {
        let remote = remote.clone();
        let connections = connections.clone();
        let event_handler = event_handler_inner.clone();
        send_channel_in.for_each(move |(id, conn_in, sink): (Id, mpsc::UnboundedReceiver<Arc<RwLock<ClientState>>>, SplitSink)| {
            let sink = Arc::new(RwLock::new(sink));
            let connections = connections.clone();
            let event_handler = event_handler.clone();
            remote.spawn(move |_| {
                conn_in.for_each(move |state: Arc<RwLock<ClientState>>| {
                    let msg = serde_json::to_string(state.read().unwrap().deref()).unwrap();
                    // println!("Sending message: {}", msg);
                    let mut sink = sink.write().unwrap();
                    let ok_send = sink.start_send(OwnedMessage::Text(msg)).is_ok();
                    let ok_poll = sink.poll_complete().is_ok();
                    if !ok_send || !ok_poll {
                        println!("Failed to send, kicking client {}", id);
                        kick(&connections, id, &*event_handler);
                    }                    
                    Ok(())
                })
            });
            Ok(())
        })
    }).map_err(|e| println!("Error while sending messages: {:?}", e));

    // Run maIn loop
    let main_fn = pool.spawn_fn(move || {
        event_handler.main_loop();
        Ok::<(), ()>(())
    }).map_err(|e| println!("Error in main callback function: {:?}", e));
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

fn build_server(handle: &Handle) -> WsServer<NoTlsAcceptor, TcpListener> {
    let address = format!("localhost:{}", read_env_var("CORE_PORT"));
    Server::bind(address, handle).expect("Failed to create server")
}

fn process_message<T>(
    id: Id,
    msg: &OwnedMessage,
    event_handler: &T,
    connections: Connections,
) where
    T: EventHandler,
{
    match *msg {
        OwnedMessage::Pong(_) => {
            
        }
        OwnedMessage::Text(ref content) => {
            let msg = Msg {
                id,
                content: content.clone(),
            };
            println!("Received message: {}", msg.content);
            event_handler.message(&msg);
        }
        OwnedMessage::Close(_) => {
            kick(&connections, id, event_handler);
        }
        _ => {}
    };
}

fn kick<T>(connections: &Connections, id: Id, event_handler: &T)
where
    T: EventHandler,
{
    connections
        .write()
        .unwrap()
        .remove(&id)
        .and_then(|_| Some(println!("Client with id {} disconnected", id)))
        .expect("Tried to remove id that was not in list");
    event_handler.disconnect(id);
}


struct IdGen {
    count: Id,
}
impl IdGen {
    fn new() -> Self {
        IdGen { count: 0 }
    }
}


impl Iterator for IdGen {
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

#[test]
fn id_gen_unique() {
    let mut id_gen = IdGen::new();
    let mut ids = Vec::new();
    let ids_count = 1000;
    for _ in 0..ids_count {
        ids.push(id_gen.next());
    }
    ids.sort();
    ids.dedup_by_key(|id| *id);
    assert_eq!(ids_count, ids.len());
}
