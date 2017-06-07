extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fmt::Debug;

use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;
use websocket::client::async::Framed;
use websocket::async::{Server, MessageCodec};
use websocket::WebSocketError;

use tokio_core::net::TcpStream;
use tokio_core::reactor::{Handle, Core};

use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};
use std::sync::{Mutex, Arc};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = Server::bind("localhost:8081", &handle).unwrap();

    let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(|(upgrade, addr)| {
            println!("Got a connection from: {}", addr);
            if !upgrade.protocols().iter().any(|s| s == "rust-websocket") {
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle);
                return Ok(());
            }

            let f = upgrade
                .use_protocol("rust-websocket")
                .accept()
                .and_then(|(server, _)| server.send(Message::text("Hello World!").into()))
                .and_then(|server| {
                    future::loop_fn(server, |stream| {
                        stream.into_future()
                            .or_else(|(err, stream)| {
                                println!("Could not send message: {:?}", err);
                                stream.send(OwnedMessage::Close(None)).map(|s| (None, s))
                            })
                            .and_then(|(msg, stream)|{
                                let mut state = State::new();
                                handle_incomming(&mut state, msg);
                                send(&state, stream)
                            })
                    })
                });
            spawn_future(f, "Client Status", &handle);
            Ok(())
        });


    core.run(f).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static,
          E: Debug
{
    handle.spawn(f.map_err(move |e| println!("{}: '{:?}'", desc, e))
        .map(move |_| println!("{}: Finished.", desc)));
}


fn handle_incomming(state: &mut State, msg: Option<OwnedMessage>) {
    /*match msg {
        Some(OwnedMessage::Text(txt)) => {
            stream.send(OwnedMessage::Text(txt))
                .map(|s| Loop::Continue(s))
                .boxed()
        }
        Some(OwnedMessage::Binary(bin)) => {
            stream.send(OwnedMessage::Binary(bin))
                .map(|s| Loop::Continue(s))
                .boxed()
        }
        Some(OwnedMessage::Ping(data)) => {
            stream.send(OwnedMessage::Pong(data))
                .map(|s| Loop::Continue(s))
                .boxed()
        }
        Some(OwnedMessage::Close(_)) => {
            stream.send(OwnedMessage::Close(None))
                .map(|_| Loop::Break(()))
                .boxed()
        }
        Some(OwnedMessage::Pong(_)) => {
            future::ok(Loop::Continue(stream)).boxed()
        }
        None => {
            future::ok(Loop::Break(())).boxed()
        },
    }*/
}

type FramedStream = Framed<TcpStream, MessageCodec<OwnedMessage>>;

fn send(state: &State, stream: FramedStream) -> Box<Future<Item=Loop<(), FramedStream>,Error=WebSocketError>> {
    stream
        .send(OwnedMessage::Text(serde_json::to_string(&state).unwrap()))
        .map(|s| {
            if shutdown() {
                Loop::Break(())
            } else {
                Loop::Continue(s)
            }
        })
        .boxed()

}

fn shutdown() -> bool {
    false
}

#[derive(Serialize)]
struct State {
    msg: String,
}

impl State {
    fn new() -> State {
        State {
            msg: "git gud".to_string()
        }
    }
}

