extern crate specs;
extern crate futures;
extern crate serde_json;
extern crate websocket_server;
extern crate byteorder;

use self::specs::{Join, WriteStorage, System, Entities, Fetch};
use self::futures::{Future, Sink};
use self::websocket_server::Message;
use self::byteorder::{BigEndian, WriteBytesExt};

use model::game::Id;
use model::comp::{Ping, Pong, Player};
use util::{timestamp, SeqIdGen};

use std::sync::RwLock;


pub struct Delay;
impl<'a> System<'a> for Delay {
    #[allow(type_complexity)]
    type SystemData = (WriteStorage<'a, Player>,
     WriteStorage<'a, Ping>,
     WriteStorage<'a, Pong>,
     Entities<'a>,
     Fetch<'a, RwLock<SeqIdGen>>);

    fn run(&mut self, (mut player, mut ping, mut pong, entities, seq_id_gen): Self::SystemData) {
        let mut pinged_players = Vec::new();
        for (mut player, entity, _) in (&mut player, &*entities, &mut ping).join() {
            send_ping(&mut player, &seq_id_gen);
            pinged_players.push(entity);
        }

        for pinged_player in pinged_players {
            ping.remove(pinged_player);
        }

        let mut ponged_players = Vec::new();
        for (mut player, entity, pong) in (&mut player, &*entities, &mut pong).join() {
            add_pong(&mut player, pong);
            ponged_players.push(entity);
        }
        for ponged_player in ponged_players {
            pong.remove(ponged_player);
        }

        for mut player in (&mut player).join() {
            manage_delays(&mut player);
        }
    }
}

fn add_pong(player: &mut Player, pong: &Pong) {
    if let Some(pingpong) = player.pingpongs.get_mut(&pong.ping_id) {
        pingpong.1 = Some(pong.timestamp);
    }
}

fn send_ping(player: &mut Player, seq_id_gen: &RwLock<SeqIdGen>) {
    let send_channel = player.send_channel.clone();
    let timestamp = timestamp();
    let id = seq_id_gen.write().unwrap().gen();
    player.pingpongs.insert(id, (timestamp, None));
    let mut id_as_bytes = vec![];
    id_as_bytes.write_u32::<BigEndian>(id).unwrap();
    send_channel
        .send(Message::Ping(id_as_bytes))
        .wait()
        .unwrap();
}

const DELAY_BUFFER_TIME: u64 = 10_000;
fn manage_delays(player: &mut Player) {
    let mut delays = Vec::new();
    let mut expired = Vec::new();
    let now = timestamp();
    for (ping_id, &timestamps) in player.pingpongs.iter() {
        let (ping_time, pong_time) = timestamps;
        if let Some(pong_time) = pong_time {
            delays.push(pong_time as usize - ping_time as usize);
            if now - pong_time > DELAY_BUFFER_TIME {
                expired.push(ping_id.clone());
            }
        }
    }
    if !delays.is_empty() {
        let sum: usize = delays.iter().sum();
        player.delay = sum / delays.len() / 2;
    }
    for ping_id in expired {
        player.pingpongs.remove(&ping_id).unwrap();
    }
}
