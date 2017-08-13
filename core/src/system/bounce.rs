extern crate specs;

use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System};

use model::comp::{Pos, Vel, Bounciness, Bounds, Actor};
use model::game::{Id, Vector};
use collision::World;
use util::angle;
use std::sync::RwLock;

pub struct Bounce;
impl<'a> System<'a> for Bounce {
    #[allow(type_complexity)]
    type SystemData = (WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     ReadStorage<'a, Actor>,
     ReadStorage<'a, Bounciness>,
     Fetch<'a, Bounds<Pos>>,
     Fetch<'a, RwLock<World<Id>>>);

    fn run(&mut self, (mut pos, mut vel, actor, bounciness, pos_bounds, world): Self::SystemData) {
        let world = world.read().unwrap();
        for (mut pos, mut vel, actor, _) in (&mut pos, &mut vel, &actor, &bounciness).join() {
            handle_movement(actor, &mut pos, &mut vel, &pos_bounds, &world);
        }
    }
}

fn handle_movement(
    actor: &Actor,
    pos: &mut Pos,
    vel: &mut Vel,
    bounds: &Bounds<Pos>,
    world: &World<Id>,
) {
    world.query_intersects_id(&actor.id, |other| {
        let own: Vector = Vector { x: pos.x, y: pos.y };
        let other = Vector {
            x: other.bounds.x,
            y: other.bounds.y,
        };
        if own == other {
            vel.x = -vel.x;
        } else {
            let angle = angle(&own, &other);
            const SPEED_UP: i32 = 1;
            let sped_up_x = vel.x.abs() + SPEED_UP;
            let sped_up_y = vel.y + vel.y / vel.y.abs() * SPEED_UP;
            if angle > 270.0 || angle < 90.0 {
                vel.x = -sped_up_x;
                vel.y = sped_up_y;
            } else {
                vel.x = sped_up_x;
                vel.y = sped_up_y;
            }
        }
    });
    let next_x = pos.x + vel.x;
    let next_y = pos.y + vel.y;
    if next_x > bounds.max.x || next_x < bounds.min.x {
        pos.x = 500;
        pos.y = 500;
        vel.x = 7;
        vel.y = 11;
    }
    if next_y > bounds.max.y || next_y < bounds.min.y {
        vel.y = -vel.y;
    }
}
