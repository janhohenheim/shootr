extern crate specs;

use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System};

use model::comp::{Pos, Vel, Bounciness, Bounds, Actor};
use model::game::Id;
use collision::World;
use std::sync::RwLock;

pub struct Bounce;
impl<'a> System<'a> for Bounce {
    type SystemData = (ReadStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     ReadStorage<'a, Actor>,
     ReadStorage<'a, Bounciness>,
     Fetch<'a, Bounds<Pos>>,
     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, (pos, mut vel, actor, bounciness, pos_bounds, world): Self::SystemData) {
        let world = world.read().unwrap();
        for (pos, mut vel, actor, _) in (&pos, &mut vel, &actor, &bounciness).join() {
            handle_movement(actor, pos, &mut vel, &pos_bounds, &world);
        }
    }
}

fn handle_movement(
    actor: &Actor,
    pos: &Pos,
    vel: &mut Vel,
    bounds: &Bounds<Pos>,
    world: &World<Id>,
) {
    world.query_intersects_id(&actor.id, |other| {
        let d_x = other.bounds.x - pos.x;
        let d_y = other.bounds.y - pos.y;
    });
    let next_x = pos.x + vel.x;
    let next_y = pos.y + vel.y;
    if next_x > bounds.max.x || next_x < bounds.min.x {
        vel.x = -vel.x;
    }

    if next_y > bounds.max.y || next_y < bounds.min.y {
        vel.y = -vel.y;
    }
}
