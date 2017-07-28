extern crate specs;
use self::specs::{Join, WriteStorage, ReadStorage, System, Entities};

use model::comp::{Pos, Vel, Acc, Friction, Connect};
use model::game::Vector;

pub struct Spawn;
impl<'a> System<'a> for Spawn {
    #[allow(type_complexity)]
    type SystemData = (Entities<'a>,
     ReadStorage<'a, Connect>,

     WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     WriteStorage<'a, Acc>,
     WriteStorage<'a, Friction>);

    fn run(
        &mut self,
        (entities, connect, mut pos, mut vel, mut acc, mut friction): Self::SystemData,
    ) {
        for (entity, _) in (&*entities, &connect).join() {
            let x = if entity.id() % 2 == 0 { 20 } else { 980 };
            acc.insert(entity, Acc::from(Vector { x: 0, y: 0 }));
            vel.insert(entity, Vel::from(Vector { x: 0, y: 0 }));
            pos.insert(entity, Pos::from(Vector { x, y: 500 }));
            friction.insert(entity, Friction(2));
        }
    }
}
