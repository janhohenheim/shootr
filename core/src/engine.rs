extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
extern crate dotenv;

use self::dotenv::dotenv;

pub use self::websocket::message::OwnedMessage;
use self::websocket::server::{WsServer, InvalidConnection};
use self::websocket::async::{Server, MessageCodec};
use self::websocket::client::async::Framed;
use self::websocket::server::NoTlsAcceptor;

use self::tokio_core::reactor::{Handle, Core};
use self::tokio_core::net::{TcpStream, TcpListener};

use self::futures::{Future, Sink, Stream};
use self::futures::sync::mpsc;
use self::futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::fmt::Debug;
use std::net::SocketAddr;

use util::read_env_var;


pub type SendChannel = mpsc::UnboundedSender<OwnedMessage>;
pub trait EventHandler {
    type Id: Send + Sync + Clone;
    fn new() -> Self;
    fn main_loop(&self);
    fn message(&self, id: Self::Id, msg: OwnedMessage);
    fn connect(&self, addr: SocketAddr, send_channel: SendChannel) -> Option<Self::Id>;
    fn disconnect(&self, id: Self::Id);
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
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
    let (send_channel_out, send_channel_in) = mpsc::unbounded();

    let event_handler = T::new();
    let event_handler = Arc::new(event_handler);

    let event_handler_inner = event_handler.clone();
    let handle_inner = handle.clone();
    // Handle new connection
    let connection_handler = server
        .incoming()
        .map(|(upgrade, addr)| Some((upgrade, addr)))
        .or_else(|InvalidConnection { error, .. }| {
            println!("Client failed to connect: {}", error);
            Ok(None)
        })
        .for_each(move |conn| {
            if conn.is_none() {
                return Ok(());
            }
            let (upgrade, addr) = conn.unwrap();
            let event_handler = event_handler_inner.clone();
            let receive_channel = receive_channel_out.clone();
            let send_channel = send_channel_out.clone();
            let f = upgrade.accept().and_then(move |(framed, _)| {
                let (conn_out, conn_in) = mpsc::unbounded();
                let res = event_handler.connect(addr, conn_out);
                if let Some(id) = res {
                    let (sink, stream) = framed.split();
                    send_channel
                        .send((id.clone(), conn_in, sink))
                        .wait()
                        .unwrap();
                    receive_channel.send((id, stream)).wait().unwrap();
                }
                Ok(())
            });
            spawn_future(f, "Handle new connection", &handle_inner);
            Ok(())
        })
        .map_err(|e: ()| e);


    // Handle receiving messages from a client
    let event_handler_inner = event_handler.clone();
    let remote_inner = remote.clone();
    let receive_handler = pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let event_handler = event_handler_inner.clone();
            remote_inner.spawn(move |_| {
                stream
                    .for_each(move |msg| {
                        let id = id.clone();
                        if let OwnedMessage::Close(_) = msg {
                            event_handler.disconnect(id);
                        } else {
                            event_handler.message(id, msg);
                        }
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
        let event_handler = event_handler_inner.clone();
        type SinkContent = Framed<TcpStream, MessageCodec<OwnedMessage>>;
        type SplitSink = futures::stream::SplitSink<SinkContent>;
        send_channel_in.for_each(
            move |(id, conn_in, sink): (T::Id,
                                        mpsc::UnboundedReceiver<OwnedMessage>,
                                        SplitSink)| {
                let sink = Arc::new(RwLock::new(sink));
                let event_handler = event_handler.clone();
                remote.spawn(move |_| {
                    conn_in.for_each(move |msg: OwnedMessage| {
                        let mut sink = sink.write().unwrap();
                        let ok_send = sink.start_send(msg).is_ok();
                        let ok_poll = sink.poll_complete().is_ok();
                        if !ok_send || !ok_poll {
                            println!("Failed to send, kicking client");
                            event_handler.disconnect(id.clone());
                        }
                        Ok(())
                    })
                });
                Ok(())
            },
        )
    }).map_err(|e| println!("Error while sending messages: {:?}", e));

    // Run main loop
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
