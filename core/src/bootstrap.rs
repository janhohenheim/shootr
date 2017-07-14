extern crate specs;

use self::specs::World;
use model::comp::*;
use model::game::Bounds;

pub fn prepare_world(world: &mut World) {
    register_components(world);
    add_constraints(world);
}

fn register_components(world: &mut World) {
    world.register::<Pos>();
    world.register::<Vel>();
    world.register::<Acc>();
    world.register::<PlayerInput>();
    world.register::<Bounciness>();
    world.register::<PlayerId>();
}

fn add_constraints(world: &mut World) {
    world.add_resource(Bounds {
        min: Acc { x: -5, y: -5 },
        max: Acc { x: 5, y: 5 },
    });
    world.add_resource(Bounds {
        min: Vel { x: -50, y: -50 },
        max: Vel { x: 50, y: 50 },
    });
    world.add_resource(Bounds {
        min: Pos { x: 0, y: 0 },
        max: Pos { x: 1000, y: 1000 },
    });
}
