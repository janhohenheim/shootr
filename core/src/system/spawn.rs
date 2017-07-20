extern crate specs;
use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System, Entities, EntitiesRes, Entity};

use model::comp::{Pos, Vel, Acc, Friction, Connect, Disconnect};
use model::game::Vector;

pub struct Spawn;
impl<'a> System<'a> for Spawn {
    type SystemData = (Entities<'a>,
     WriteStorage<'a, Connect>,
     ReadStorage<'a, Disconnect>,

     WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     WriteStorage<'a, Acc>,
     WriteStorage<'a, Friction>);

fn run(&mut self,
( mut entities, mut connect,  disconnect,
mut pos, mut vel, mut acc, mut friction): Self::SystemData){
        let mut to_spawn = Vec::new();
        for (entity, _) in (&*entities, &connect).join() {
            to_spawn.push(entity);
        }
        for entity in to_spawn {
            let x = if entity.id() % 2 == 0 { 20 } else { 980 };
            acc.insert(entity, Acc::from(Vector { x: 0, y: 0 }));
            vel.insert(entity, Vel::from(Vector { x: 0, y: 0 }));
            pos.insert(entity, Pos::from(Vector { x, y: 500 }));
            friction.insert(entity, Friction(2));
            connect.remove(entity);
        }

        let mut to_despawn = Vec::new();
        for (entity, _) in (&*entities, &disconnect).join() {
            to_despawn.push(entity);
        }
        for entity in to_despawn.drain(..) {
            entities.delete(entity);
        }
    }
}
