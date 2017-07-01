extern crate specs;

use self::specs::{Component, VecStorage};

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
