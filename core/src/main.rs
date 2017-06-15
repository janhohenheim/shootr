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

use tokio_core::reactor::{Handle, Core};

use futures::{Future, Sink, Stream};
use futures::future::{self, Loop};
use futures_cpupool::CpuPool;

use std::sync::{RwLock, Arc};
use std::thread;
use std::rc::Rc;
use std::fmt::Debug;
use std::time::Duration;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("localhost:8081", &handle).unwrap();
    let pool = Rc::new(CpuPool::new_num_cpus());
    let state = Arc::new(RwLock::new(State::new()));
    let (read_channel_out, read_channel_in) = futures::sync::mpsc::unbounded();
    let (write_channel_out, write_channel_in) = futures::sync::mpsc::unbounded();
    let handle_inner = handle.clone();
    let connection_handler = server.incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            println!("Got a connection from: {}", addr);
            if !upgrade.protocols().iter().any(|s| s == "rust-websocket") {
                spawn_future(upgrade.reject(), "Upgrade Rejection", &handle_inner);
                return Ok(());
            }
            let read_channel_out = read_channel_out.clone();
            let write_channel_out = write_channel_out.clone();
            upgrade
                .use_protocol("rust-websocket")
                .accept()
                .and_then(move |(framed, _)| {
                    let (sink, stream) = framed.split();
                    read_channel_out.send(stream).wait().unwrap();
                    write_channel_out.send(sink).wait().unwrap();
                    Ok(())
                })
                .wait()
                .unwrap();
            Ok(())
        })
        .map_err(|_| ());


    let state_read = state.clone();
    let remote_write = remote.clone();
    let read_handler = pool.spawn_fn( || {
        read_channel_in.for_each(move |stream| {
            let state_read = state_read.clone();
            remote_write.spawn(|_| {
                stream
                    .for_each(move |msg|{
                        handle_incoming(&mut state_read.write().unwrap(), &msg);
                        Ok(())
                    })
                    .map_err(|_| ())
            });
            Ok(())
        })
    });

    let write_handler = pool.spawn_fn(move || {
        write_channel_in.for_each(move |sink| {
            let state_write = state.clone();
            future::loop_fn(sink, move |sink| {
                thread::sleep(Duration::from_millis(100));
                send(&state_write.read().unwrap(), sink)
                    .map(|sink| Loop::Continue(sink))
                    .map_err(|_| Loop::Break::<(), SplitSink>(()))
            }).map_err(|_| ())
        })
    });

    let handlers = connection_handler.select2(read_handler.select(write_handler));
    core.run(handlers).map_err(|_| println!("err")).unwrap();
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
    where F: Future<Item = I, Error = E> + 'static,
          E: Debug
{
    handle.spawn(f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
                     .map(move |_| println!("{}: Finished.", desc)));
}


fn handle_incoming(state: &mut State, msg: &OwnedMessage) {
    match msg {
        &OwnedMessage::Text(ref txt) => {
            println!("Received message: {}", txt);
            state.msg = txt.clone();
        }
        _ => {}
    }
}

type SplitSink = futures::stream::SplitSink<
                    websocket::client::async::Framed<tokio_core::net::TcpStream,
                    websocket::async::MessageCodec<websocket::OwnedMessage>>
                >;

fn send(state: &State, sink: SplitSink) -> futures::sink::Send<SplitSink> {
    let msg = serde_json::to_string(&state).unwrap();
    println!("Sending message: {}", msg);
    sink.send(OwnedMessage::Text(msg))
}


#[derive(Serialize)]
struct State {
    msg: String,
}

impl State {
    fn new() -> State {
        State { msg: "git gud".to_string() }
    }
}
