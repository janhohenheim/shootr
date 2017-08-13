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
    Spawn,
    Despawn,
    WorldUpdate,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerMsg<T>
where
    T: Serialize + Debug,
{
    pub opcode: OpCode,
    pub payload: T,
}
impl ServerMsg<Vec<Value>> {
    pub fn new_greeting(own_id: &Id, actors: &[&Actor]) -> Self {
        ServerMsg {
            opcode: OpCode::Greeting,
            payload: vec![json!(own_id), json!(actors)],
        }
    }
}
impl ServerMsg<Value> {
    pub fn new_spawn(new_actor: &Actor) -> Self {
        ServerMsg {
            opcode: OpCode::Spawn,
            payload: json!(new_actor),
        }
    }
    pub fn new_despawn(id: &Id) -> Self {
        ServerMsg {
            opcode: OpCode::Despawn,
            payload: json!(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Command {
    MoveUp,
    MoveDown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientMsg {
    pub id: u32,
    pub command: Command,
    pub active: bool,
}
