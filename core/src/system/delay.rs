extern crate specs;
extern crate futures;
extern crate serde_json;
extern crate websocket_server;

use self::specs::{Join, WriteStorage, System, Entities, Fetch};
use self::futures::{Future, Sink};
use self::websocket_server::Message;

use model::client::Message as ClientMessage;
use model::comp::{Ping, Pong, Player, Delay as PlayerDelay};
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
        pingpong.1 = Some(pong.timestamps.clone());
    }
}

fn send_ping(player: &mut Player, seq_id_gen: &RwLock<SeqIdGen>) {
    let send_channel = player.send_channel.clone();
    let timestamp = timestamp();
    let id = seq_id_gen.write().unwrap().gen();
    player.pingpongs.insert(id, (timestamp, None));
    let msg = ClientMessage::new_ping(id);
    let msg = serde_json::to_string(&msg).expect(&format!("Failed to serialize object {:?}", msg));
    send_channel.send(Message::Text(msg)).wait().unwrap();
}

const DELAY_BUFFER_TIME: u64 = 10_000;
fn manage_delays(player: &mut Player) {
    let mut delays = Vec::new();
    let mut expired = Vec::new();
    let now = timestamp();
    for (ping_id, ref timestamps) in &player.pingpongs {
        let &&(ref ping_time, ref pong_time) = timestamps;
        if let Some(ref pong_time) = *pong_time {
            let server_time = pong_time.server;
            let client_time = pong_time.client;
            let ping = (server_time - *ping_time) as usize / 2;
            let delay = PlayerDelay {
                ping,
                clock: (client_time as i32 - ping as i32 - server_time as i32),
            };
            delays.push(delay);
            if now - server_time > DELAY_BUFFER_TIME {
                expired.push(*ping_id);
            }
        }
    }
    if !delays.is_empty() {
        let ping_sum: usize = delays.iter().map(|delay| delay.ping).sum();
        let clock_sum: i32 = delays.iter().map(|delay| delay.clock).sum();
        player.delay.ping = ping_sum / delays.len();
        player.delay.clock = clock_sum / delays.len() as i32;
    }
    for ping_id in expired {
        player.pingpongs.remove(&ping_id).unwrap();
    }
}
