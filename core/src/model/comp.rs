extern crate specs;
use self::specs::{Component, VecStorage};

use engine::Id;
use super::game::Vector;
use std::ops::{Deref, DerefMut};
use std::convert::From;

macro_rules! vectype {
    (  $name:ident ) => {
        newtype!(
            $name(Vector): Debug,
            Clone,
            PartialEq,
            Eq,
            Serialize,
            Component
        );
    };
}

vectype!(Acc);
vectype!(Vel);
vectype!(Pos);


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
