#[derive(Debug, Clone, Serialize)]
pub struct KeyState {
    pub pressed: bool,
    pub fired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bounds<T> {
    pub max: T,
    pub min: T,
}