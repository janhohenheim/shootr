use ecs::comp::{Pos, Vel};

#[derive(Serialize)]
pub struct ClientState {
    pub pos: Pos,
    pub vel: Vel,
    // UTC timestamp in millis
    pub timestamp: u64,
}

#[derive(Serialize, Debug)]
pub enum Axis {
    X,
    Y,
}
