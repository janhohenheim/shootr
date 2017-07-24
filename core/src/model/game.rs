extern crate uuid;
use self::uuid::Uuid;

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
