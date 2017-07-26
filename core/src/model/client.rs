extern crate serde;
extern crate serde_json;

use self::serde::ser::Serialize;
use self::serde_json::Value;
use model::game::Id;
use model::comp::Actor;
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
impl Message<Vec<Value>> {
    pub fn new_greeting(own_id: &Id, actors: &Vec<&Actor>) -> Self {
        Message {
            opcode: OpCode::Greeting,
            payload: vec![json!(own_id), json!(actors)],
        }
    }
}
impl Message<Value> {
    pub fn new_connection(new_actor: &Actor) -> Self {
        Message {
            opcode: OpCode::Connect,
            payload: json!(new_actor),
        }
    }
    pub fn new_disconnect(id: &Id) -> Self {
        Message {
            opcode: OpCode::Disconnect,
            payload: json!(id),
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
    pub id: u32,
    pub key: Key,
    pub pressed: bool,
}
