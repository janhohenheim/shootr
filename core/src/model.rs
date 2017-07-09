use ecs::comp::{Pos, Vel};

#[derive(Serialize)]
pub struct ClientState {
    pub pos: Pos,
    pub vel: Vel,
}

#[derive(Serialize, Debug)]
pub enum Axis {
    X,
    Y,
}
