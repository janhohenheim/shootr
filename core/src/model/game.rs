extern crate specs;
extern crate uuid;

use self::specs::Entity;
use self::uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};



pub type Id = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Add, AddAssign)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

#[macro_export]
macro_rules! vectype {
    (  $name:ident ) => {
        newtype!(
            $name(Vector): Debug,
            Clone,
            PartialEq,
            Eq,
            Serialize,
            Add,
            AddAssign,
            Component
        );
    };
}
