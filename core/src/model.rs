use ::ecs::comp::Pos;
use ::ecs::res::TimeProgress;

#[derive(Serialize)]
pub struct ClientState {
    pub pos: Pos,
    pub progress: TimeProgress,
}

#[derive(Serialize, Debug)]
pub enum Axis {
    X, Y
}