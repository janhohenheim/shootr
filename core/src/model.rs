extern crate specs;

use self::specs::{Component, VecStorage};

use engine::Id;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ClientState {
    pub pos: Pos,
    pub vel: Vel,
    // UTC timestamp in ms
    pub timestamp: u64,
}

type IdsType = Arc<RwLock<Vec<Id>>>;
#[derive(Clone, Debug)]
pub struct Ids(pub IdsType);
impl Deref for Ids {
    type Target = IdsType;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Ids {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Acc {
    pub x: i32,
    pub y: i32,
}
impl Component for Acc {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize)]
pub struct Vel {
    pub x: i32,
    pub y: i32,
}
impl Component for Vel {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Key {
    ArrowUp,
    ArrowDown,
}


#[derive(Debug, Clone, Deserialize)]
pub struct InputMsg {
    pub id: i64,
    pub key: Key,
    pub pressed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyState {
    pub pressed: bool,
    pub fired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerInput {
    pub key_states: HashMap<Key, KeyState>,
}
impl Component for PlayerInput {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize)]
pub struct BallAiInput {}
impl Component for BallAiInput {
    type Storage = VecStorage<Self>;
}
