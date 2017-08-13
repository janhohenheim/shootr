extern crate specs;
extern crate websocket_server;
use self::specs::{Component, DenseVecStorage};
use self::websocket_server::SendChannel;

use super::game::{Vector, Id as GameId};
use model::network::Command;
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


pub type KeyboardState = HashMap<Command, bool>;
#[derive(Component)]
pub struct Player {
    pub send_channel: SendChannel,
    pub inputs: Vec<KeyboardState>,
    pub last_input: u32,
}

impl Player {
    pub fn new(send_channel: SendChannel) -> Self {
        Player {
            send_channel,
            inputs: Vec::new(),
            last_input: 0,
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
