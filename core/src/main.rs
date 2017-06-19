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
use std::ops::Deref;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let remote = core.remote();
    let server = Server::bind("localhost:8081", &handle).unwrap();
    let pool = Rc::new(CpuPool::new_num_cpus());
    let connections = Arc::new(RwLock::new(Vec::new()));
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

    let connections_inner = connections.clone();
    let write_handler = pool.spawn_fn(move || {
        write_channel_in.for_each(move |sink| {
            let (write_out, write_in) = futures::sync::mpsc::unbounded();
            connections_inner.write().unwrap().push(write_out);
            remote.spawn(move |_| {
                let send = sink.send(OwnedMessage::Text("Connection established".to_owned()));
                write_in.fold(send, move |send, state:Arc<RwLock<State>>|{
                    let msg = serde_json::to_string(&state.read().unwrap().deref()).unwrap();
                    println!("Sending message: {}", msg);
                    let send = send.wait().unwrap().send(OwnedMessage::Text(msg));
                    Ok(send)
                }).map(|_| ())
            });
            Ok(())
        })
    });                

    let game_loop = pool.spawn_fn(move || {
         future::loop_fn(connections, move |connections| {
                thread::sleep(Duration::from_millis(100));
                for conn in connections.write().unwrap().iter() {
                    let state = state.clone();
                    conn.send(state).unwrap();
                }
                match 1 {
                    1=>Ok(Loop::Continue(connections)),
                    2=>Ok(Loop::Break(())),
                    _=>Err(())
                }
        })
    });

    let handlers = game_loop.select2(connection_handler.select2(read_handler.select(write_handler)));
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


#[derive(Serialize)]
struct State {
    msg: String,
}

impl State {
    fn new() -> State {
        State { msg: "git gud".to_string() }
    }
}
