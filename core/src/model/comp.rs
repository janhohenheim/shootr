extern crate specs;
extern crate websocket_server;
use self::specs::{Component, DenseVecStorage};
use self::websocket_server::SendChannel;

use super::game::{Vector, Id as GameId};
use util::SeqId as PingId;
use model::client::Key;
use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::collections::HashMap;

vectype!(Acc);
vectype!(Vel);
vectype!(Pos);


#[derive(Debug, Clone, Serialize, Component)]
pub struct Bounciness {}

newtype!(Friction(i32): Debug, Clone, Serialize, Component);

#[derive(Debug, Clone, Serialize)]
pub enum ActorKind {
    Player,
    Ball,
}


#[derive(Debug, Clone, Serialize, Component)]
pub struct Actor {
    pub id: GameId,
    pub kind: ActorKind,
}

#[derive(Component)]
pub struct ToSpawn;
#[derive(Component)]
pub struct ToDespawn;
#[derive(Component)]
pub struct Ping;
#[derive(Component)]
pub struct Pong {
    pub ping_id: PingId,
    pub timestamps: PongTimestamps,
}

#[derive(Debug, Clone)]
pub struct PongTimestamps {
    pub server: u64,
    pub client: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Delay {
    pub ping: usize,
    pub clock: i32,
}

pub type KeyboardState = HashMap<Key, bool>;
#[derive(Component)]
pub struct Player {
    pub send_channel: SendChannel,
    pub inputs: Vec<KeyboardState>,
    pub last_input: u32,
    pub delay: Delay,
    pub pingpongs: HashMap<PingId, (u64, Option<PongTimestamps>)>,
}
impl Player {
    pub fn new(send_channel: SendChannel) -> Self {
        Player {
            send_channel,
            inputs: Vec::new(),
            last_input: 0,
            delay: Delay { ping: 0, clock: 0 },
            pingpongs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Bounds<T> {
    pub max: T,
    pub min: T,
}
impl<T> Component for Bounds<T>
where
    T: Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}
