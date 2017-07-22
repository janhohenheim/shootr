use super::comp::{Pos, Vel, Acc};
use super::game::Id;


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
