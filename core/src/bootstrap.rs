extern crate specs;

use self::specs::World;
use model::comp::*;
use model::game::Vector;
use util::SeqIdGen;

use std::sync::RwLock;

pub fn prepare_world(world: &mut World) {
    register_components(world);
    add_constraints(world);
    add_utils(world);
}


macro_rules! register {
    (  $world:ident: $( $comp:ty, )+  ) => {
        $ (
            $world.register::<$comp>();
        )+
    };
}

fn register_components(world: &mut World) {
    register!(
        world: Pos,
        Vel,
        Acc,
        Bounciness,
        Player,
        Friction,
        ToSpawn,
        ToDespawn,
        Actor,
    );
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

fn add_utils(world: &mut World) {
    world.add_resource(RwLock::new(SeqIdGen::default()));
}
