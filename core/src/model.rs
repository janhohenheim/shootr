extern crate specs;

use self::specs::{Component, VecStorage};

use engine::Id;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};


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

#[derive(Debug, Clone, Deserialize)]
pub enum Key {
    ArrowLeft,
    ArrowRight,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub id: i64,
    pub key: Key,
    pub state: bool
}
