use super::comp::{Pos, Vel, Acc};
use engine::Id;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ClientState {
    pub ball: Ball,
    pub players: HashMap<Id, Player>,
    // UTC timestamp in ms
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ball {
    pub pos: Pos,
    pub vel: Vel,
}

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub pos: Pos,
    pub vel: Vel,
    pub acc: Acc,
}


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Key {
    ArrowUp,
    ArrowDown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputMsg {
    pub id: i64,
    pub key: Key,
    pub pressed: bool,
}
