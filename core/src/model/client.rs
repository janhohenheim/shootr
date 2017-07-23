extern crate serde;

use self::serde::ser::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum OpCode {
    Greeting,
    Connect,
    Disconnect,
    WorldUpdate,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message<T>
where
    T: Serialize,
{
    pub opcode: OpCode,
    pub payload: T,
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
