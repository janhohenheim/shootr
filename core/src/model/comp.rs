extern crate specs;
extern crate uuid;

use self::specs::{Component, VecStorage};
use self::uuid::Uuid;

use engine::SendChannel;
use super::client::Key;
use super::game::{Vector, KeyState};
use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::collections::HashMap;

vectype!(Acc);
vectype!(Vel);
vectype!(Pos);


#[derive(Debug, Clone, Serialize, Component)]
pub struct Bounciness {}

newtype!(Friction(i32): Debug, Clone, Serialize, Component);

#[derive(Component)]
pub struct Connect;
#[derive(Component)]
pub struct Disconnect;


pub type InputState = HashMap<Key, KeyState>;
#[derive(Component)]
pub struct Player {
    pub id: Uuid,
    pub send_channel: SendChannel,
    pub inputs: Vec<InputState>,
    pub pingpongs: HashMap<u64, (u64, Option<u64>)>,
}
impl Player {
    pub fn new(id: Uuid, send_channel: SendChannel) -> Self {
        Player {
            id,
            send_channel,
            inputs: Vec::new(),
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
    type Storage = VecStorage<Self>;
}
