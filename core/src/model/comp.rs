extern crate specs;
use self::specs::{Component, VecStorage, Entity};

use engine::SendChannel;
use super::game::Vector;
use std::ops::{Deref, DerefMut};
use std::convert::From;

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

#[derive(Component)]
pub struct Player {
    pub send_channel: SendChannel,
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
