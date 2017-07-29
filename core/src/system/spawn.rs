extern crate specs;
use self::specs::{Join, WriteStorage, ReadStorage, System, Entities, Fetch};

use model::comp::{Actor, ActorKind, Pos, Vel, Acc, Friction, Connect};
use model::game::{Vector, Id};
use collision::{World, Bounds};
use std::sync::RwLock;

pub struct Spawn;
impl<'a> System<'a> for Spawn {
    #[allow(type_complexity)]
    type SystemData = (Entities<'a>,
     ReadStorage<'a, Connect>,
     ReadStorage<'a, Actor>,

     WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     WriteStorage<'a, Acc>,
     WriteStorage<'a, Friction>,

     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities, connect, actor, mut pos, mut vel, mut acc, mut friction, world) = data;
        for (entity, actor, _) in (&*entities, &actor, &connect).join() {
            let x = if entity.id() % 2 == 0 { 20 } else { 980 };
            acc.insert(entity, Acc::from(Vector { x: 0, y: 0 }));
            vel.insert(entity, Vel::from(Vector { x: 0, y: 0 }));
            pos.insert(entity, Pos::from(Vector { x, y: 500 }));
            friction.insert(entity, Friction(2));
            let bounds = match actor.kind {
                ActorKind::Player => Bounds {
                    x,
                    y: 500,
                    width: 10,
                    height: 50,
                },
                ActorKind::Ball => Bounds {
                    x: 500,
                    y: 500,
                    width: 10,
                    height: 10,
                },
            };
            world.write().unwrap().insert(actor.id, bounds);
        }
    }
}
