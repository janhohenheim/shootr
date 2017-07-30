extern crate specs;
use self::specs::{Join, ReadStorage, System, Entities, Fetch};

use model::comp::{ToDespawn, Actor};
use model::game::Id;
use collision::World;
use std::sync::RwLock;

pub struct Despawn;
impl<'a> System<'a> for Despawn {
    type SystemData = (Entities<'a>,
     ReadStorage<'a, Actor>,
     ReadStorage<'a, ToDespawn>,
     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, (entities, actor, despawn, world): Self::SystemData) {
        let mut to_despawn = Vec::new();
        let mut world = world.write().unwrap();
        for (entity, actor, _) in (&*entities, &actor, &despawn).join() {
            to_despawn.push(entity);
            world.remove(&actor.id);
        }
        for entity in to_despawn.drain(..) {
            entities.delete(entity);
        }
    }
}
