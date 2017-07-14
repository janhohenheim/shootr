use super::comp::{Pos, Vel};

#[derive(Debug, Clone, Serialize)]
pub struct ClientState {
    pub pos: Pos,
    pub vel: Vel,
    // UTC timestamp in ms
    pub timestamp: u64,
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