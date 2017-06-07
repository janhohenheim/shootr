extern crate websocket;
extern crate futures;
extern crate tokio_core;

use std::fmt::Debug;

use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;
use websocket::client::async::Framed;
use websocket::async::Server;
use websocket::async::futures::stream::Map;

use tokio_core::reactor::{Handle, Core};
use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    // bind to the server
    let server = Server::bind("localhost:8081", &handle).unwrap();

    // time to build the server's future
    // this will be a struct containing everything the server is going to do

    // a stream of incoming connections
    let f = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(|(upgrade, addr)| {
            println!("Got a connection from: {}", addr);
            // check if it has the protocol we want
            if !upgrade.protocols().iter().any(|s| s == "rust-websocket") {
                // reject it if it doesn't
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle);
                return Ok(());
            }

            // accept the request to be a ws connection if it does
            let f = upgrade
                .use_protocol("rust-websocket")
                .accept()
                // send a greeting!
                .and_then(|(server, _)| server.send(Message::text("Hello World!").into()))
                // simple echo server impl
                .and_then(|server| {
                    future::loop_fn(server, |stream| {
                        stream.into_future()
                            .or_else(|(err, stream)| {
                                println!("Could not send message: {:?}", err);
                                stream.send(OwnedMessage::Close(None)).map(|s| (None, s))
                            })
                            .and_then(|(msg, stream)|{
                                handle_incomming(&msg);
                                stream.send(OwnedMessage::Text(state()))
                                    .map(|s| {
                                        if shutdown() {
                                            Loop::Continue(s)
                                        } else {
                                            Loop::Break(())
                                        }
                                    })
                                    .boxed()
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

fn handle_incomming(msg: &Option<OwnedMessage>) {
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

fn send() {
    /*
    stream
        .send(OwnedMessage::Text(state()))
        .map(|s| {
            if shutdown() {
                Loop::Continue(s)
            } else {
                Loop::Break(())
            }
        })
        .boxed()
    */
}

fn shutdown() -> bool {
    false
}


fn state() -> String {
    "ayyyy".to_string()
}