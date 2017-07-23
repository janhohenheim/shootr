extern crate serde;

use self::serde::ser::Serialize;
use super::comp::{Pos, Vel, Acc};
use super::game::Id as GameId;

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

/*
#[derive(Debug, Clone, Serialize)]
pub struct Greeting {
    pub client_id: Id,
    pub ball: Ball,
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub player_id: Id,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}


#[derive(Debug, Clone, Serialize)]
pub struct ClientState {
    pub ball: Ball,
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ball {
    pub pos: Pos,
    pub vel: Vel,
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub id: Id,
    pub delay: usize,
    pub pos: Pos,
    pub vel: Vel,
    pub acc: Acc,
}
*/
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
