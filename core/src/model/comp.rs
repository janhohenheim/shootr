extern crate specs;
use self::specs::{Component, VecStorage};

use engine::Id;
use super::game::Vector;
use std::ops::{Deref, DerefMut};
use std::convert::From;

vectype!(Acc);
vectype!(Vel);
vectype!(Pos);


#[derive(Debug, Clone, Serialize, Component)]
pub struct Bounciness {}

newtype!(PlayerId(Id): Debug, Clone, Serialize, Component);
newtype!(Friction(i32): Debug, Clone, Serialize, Component);


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
