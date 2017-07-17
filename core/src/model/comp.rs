extern crate specs;
use self::specs::{Component, VecStorage};

use std::ops::Deref;
use engine::Id;
use super::game::Vector;


newtype!(
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Acc(Vector): Deref, DerefMut, From, Into, Add
);
impl Component for Acc {
    type Storage = VecStorage<Self>;
}


newtype!(
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Vel(Vector): Deref, DerefMut, From, Into, Add
);
impl Component for Vel {
    type Storage = VecStorage<Self>;
}

newtype!(
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Pos(Vector): Deref, DerefMut, From, Into, Add
);
impl Component for Pos {
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
