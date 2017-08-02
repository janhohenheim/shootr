extern crate specs;
use self::specs::{Join, ReadStorage, System, Entities, Fetch};

use model::comp::{ToDespawn, Actor};
use model::game::Id;
use collision::World;
use std::sync::RwLock;

pub struct Despawn;
impl<'a> System<'a> for Despawn {
    #[allow(type_complexity)]
    type SystemData = (Entities<'a>,
     ReadStorage<'a, Actor>,
     ReadStorage<'a, ToDespawn>,
     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, (entities, actor, despawn, world): Self::SystemData) {
        let mut world = world.write().unwrap();
        for (entity, actor, _) in (&*entities, &actor, &despawn).join() {
            entities.delete(entity);
            world.remove(&actor.id);
        }
    }
}
