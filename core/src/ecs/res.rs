use engine::Id;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};


type IdsType = Arc<RwLock<Vec<Id>>>;
#[derive(Clone, Debug)]
pub struct Ids(pub IdsType);
impl Deref for Ids {
    type Target = IdsType;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Ids {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
