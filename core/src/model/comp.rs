extern crate specs;
use self::specs::{Component, VecStorage};

use engine::Id;
use super::game::Vector;
use std::ops::{Deref, DerefMut};
use std::convert::From;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Acc(Vector);
impl Deref for Acc {
    type Target = Vector;

    fn deref(&self) -> &Vector {
        &self.0
    }
}
impl DerefMut for Acc {
    fn deref_mut(&mut self) -> &mut Vector {
        &mut self.0
    }
}
impl From<Vector> for Acc {
    fn from(vector: Vector) -> Self {
        Acc(vector)
    }
}
impl Component for Acc {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Vel(Vector);
impl Deref for Vel {
    type Target = Vector;

    fn deref(&self) -> &Vector {
        &self.0
    }
}
impl DerefMut for Vel {
    fn deref_mut(&mut self) -> &mut Vector {
        &mut self.0
    }
}
impl From<Vector> for Vel {
    fn from(vector: Vector) -> Self {
        Vel(vector)
    }
}

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pos(Vector);
impl Deref for Pos {
    type Target = Vector;

    fn deref(&self) -> &Vector {
        &self.0
    }
}
impl DerefMut for Pos {
    fn deref_mut(&mut self) -> &mut Vector {
        &mut self.0
    }
}
impl From<Vector> for Pos {
    fn from(vector: Vector) -> Self {
        Pos(vector)
    }
}

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
