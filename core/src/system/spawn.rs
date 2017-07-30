extern crate specs;
use self::specs::{Join, WriteStorage, ReadStorage, System, Entities, Entity, Fetch};

use model::comp::{Actor, ActorKind, Pos, Vel, Acc, Friction, ToSpawn, Bounciness};
use model::game::{Vector, Id};
use collision::{World, Bounds};
use std::sync::RwLock;

pub struct Spawn;
impl<'a> System<'a> for Spawn {
    #[allow(type_complexity)]
    type SystemData = (Entities<'a>,
     ReadStorage<'a, ToSpawn>,
     ReadStorage<'a, Actor>,

     WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     WriteStorage<'a, Acc>,
     WriteStorage<'a, Bounciness>,
     WriteStorage<'a, Friction>,

     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, data: Self::SystemData) {
        let (entities,
             to_spawn,
             actor,
             mut pos,
             mut vel,
             mut acc,
             mut bounciness,
             mut friction,
             world) = data;
        let mut world = world.write().unwrap();
        for (entity, actor, _) in (&*entities, &actor, &to_spawn).join() {
            println!("Spawning: {:?}", actor);
            match actor.kind {
                ActorKind::Player => {
                    spawn_player(
                        entity,
                        &actor,
                        &mut acc,
                        &mut vel,
                        &mut pos,
                        &mut friction,
                        &mut world,
                    )
                }
                ActorKind::Ball => {
                    spawn_ball(
                        entity,
                        &actor,
                        &mut vel,
                        &mut pos,
                        &mut bounciness,
                        &mut world,
                    )
                }
            }
        }
    }
}

fn spawn_player(
    entity: Entity,
    actor: &Actor,
    acc: &mut WriteStorage<Acc>,
    vel: &mut WriteStorage<Vel>,
    pos: &mut WriteStorage<Pos>,
    friction: &mut WriteStorage<Friction>,
    world: &mut World<Id>,
) {
    let x = if entity.id() % 2 == 0 { 20 } else { 980 };
    let y = 500;
    acc.insert(entity, Acc::from(Vector { x: 0, y: 0 }));
    vel.insert(entity, Vel::from(Vector { x: 0, y: 0 }));
    pos.insert(entity, Pos::from(Vector { x, y }));
    friction.insert(entity, Friction(2));
    let bounds = Bounds {
        x,
        y,
        width: 10,
        height: 50,
    };
    world.add(actor.id, bounds);
}

fn spawn_ball(
    entity: Entity,
    actor: &Actor,
    vel: &mut WriteStorage<Vel>,
    pos: &mut WriteStorage<Pos>,
    bounciness: &mut WriteStorage<Bounciness>,
    world: &mut World<Id>,
) {
    let x = 500;
    let y = 500;
    vel.insert(entity, Vel::from(Vector { x: 10, y: 15 }));
    pos.insert(entity, Pos::from(Vector { x, y: 500 }));
    bounciness.insert(entity, Bounciness {});
    let bounds = Bounds {
        x,
        y,
        width: 10,
        height: 10,
    };
    println!("hai");
    world.add(actor.id, bounds);
}
