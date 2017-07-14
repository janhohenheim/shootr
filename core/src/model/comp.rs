extern crate specs;
use self::specs::{Component, VecStorage};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::ops::Deref;

use super::game::KeyState;
use super::client::Key;
use engine::Id;


#[derive(Debug, Clone, Serialize, Add, AddAssign)]
pub struct Acc {
    pub x: i32,
    pub y: i32,
}
impl Component for Acc {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize, Add, AddAssign)]
pub struct Vel {
    pub x: i32,
    pub y: i32,
}
impl Component for Vel {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize, Add, AddAssign)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}
impl Component for Pos {
    type Storage = VecStorage<Self>;
}


pub type PlayerInputMap = Arc<RwLock<HashMap<Id, RwLock<PlayerInput>>>>;
#[derive(Debug, Clone, Serialize)]
pub struct PlayerInput {
    pub key_states: HashMap<Key, KeyState>,
}
impl Component for PlayerInput {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize)]
pub struct Bounciness {}
impl Component for Bounciness {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Serialize)]
pub struct PlayerId(pub Id);
impl Component for PlayerId {
    type Storage = VecStorage<Self>;
}
impl Deref for PlayerId {
    type Target = Id;

    fn deref(&self) -> &Id {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Friction(pub i32);
impl Component for Friction {
    type Storage = VecStorage<Self>;
}
impl Deref for Friction {
    type Target = i32;

    fn deref(&self) -> &i32 {
        &self.0
    }
}