extern crate serde;
extern crate serde_json;

use self::serde::ser::Serialize;
use model::game::Id;
use model::comp::{Actor};
use std::fmt::Debug;

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
    T: Serialize + Debug,
{
    pub opcode: OpCode,
    pub payload: T,
}
impl<T> Message<T> where T: Serialize + Debug {
    pub fn new_greeting(own_id: &Id, actors: &Vec<&Actor>) -> Message<Vec<self::serde_json::Value>> {
        let mut payload = Vec::new();
        payload.push(json!(own_id));
        payload.push(json!(actors));
        Message {
            opcode: OpCode::Greeting,
            payload: payload
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Key {
    ArrowUp,
    ArrowDown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyState {
    pub id: i32,
    pub key: Key,
    pub pressed: bool,
}
