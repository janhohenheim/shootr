extern crate specs;
use self::specs::{Component, VecStorage};
use std::collections::HashMap;
use super::game::KeyState;
use super::client::Key;


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
