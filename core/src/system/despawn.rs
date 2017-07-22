extern crate specs;
use self::specs::{Join, ReadStorage, System, Entities};

use model::comp::Disconnect;

pub struct Despawn;
impl<'a> System<'a> for Despawn {
    type SystemData = (Entities<'a>, ReadStorage<'a, Disconnect>);

    fn run(&mut self, (entities, disconnect): Self::SystemData) {
        let mut to_despawn = Vec::new();
        for (entity, _) in (&*entities, &disconnect).join() {
            to_despawn.push(entity);
        }
        for entity in to_despawn.drain(..) {
            entities.delete(entity);
        }
    }
}
