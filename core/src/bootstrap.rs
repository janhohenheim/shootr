extern crate specs;

use self::specs::World;
use model::comp::*;
use model::game::Vector;

pub fn prepare_world(world: &mut World) {
    register_components(world);
    add_constraints(world);
}

fn register_components(world: &mut World) {
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Acc>();
    world.register::<Bounciness>();
    world.register::<PlayerId>();
    world.register::<Friction>();
}

fn add_constraints(world: &mut World) {
    world.add_resource(Bounds {
        min: Acc::from(Vector { x: -5, y: -5 }),
        max: Acc::from(Vector { x: 5, y: 5 }),
    });
    world.add_resource(Bounds {
        min: Vel::from(Vector { x: -50, y: -50 }),
        max: Vel::from(Vector { x: 50, y: 50 }),
    });
    world.add_resource(Bounds {
        min: Pos::from(Vector { x: 0, y: 0 }),
        max: Pos::from(Vector { x: 1000, y: 1000 }),
    });
}
