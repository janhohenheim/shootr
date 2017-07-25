extern crate serde;

use self::serde::ser::Serialize;
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize)]
pub enum OpCode {
    Greeting,
    Connect,
    Disconnect,
    WorldUpdate,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message<'a, T>
where
    T: Serialize + Debug + 'a,
{
    pub opcode: &'a OpCode,
    pub timestamp: u64,
    pub payload: &'a T,
}


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Key {
    ArrowUp,
    ArrowDown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyState {
    pub key: Key,
    pub pressed: bool,
}
