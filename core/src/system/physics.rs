extern crate specs;

use self::specs::{Join, WriteStorage, ReadStorage, System, Fetch};

use model::comp::{Pos, Vel, Acc, Bounds, Friction, Actor};
use model::game::Id;
use util::clamp;
use collision::World;
use std::sync::RwLock;

pub struct Physics;
impl<'a> System<'a> for Physics {
    #[allow(type_complexity)]
    type SystemData = (WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, Friction>,
     ReadStorage<'a, Actor>,
     Fetch<'a, Bounds<Vel>>,
     Fetch<'a, Bounds<Pos>>,
     Fetch<'a, RwLock<World<Id>>>);

    fn run(
        &mut self,
        (mut pos, mut vel, acc, friction, actor, vel_bounds, pos_bounds, world): Self::SystemData,
    ) {
        for (mut vel, acc) in (&mut vel, &acc).join() {
            vel.x = clamp(vel.x + acc.x, vel_bounds.min.x, vel_bounds.max.x);
            vel.y = clamp(vel.y + acc.y, vel_bounds.min.y, vel_bounds.max.y);
        }

        for (mut vel, friction) in (&mut vel, &friction).join() {
            if vel.y == 0 {
                continue;
            }
            let sign = if vel.y < 0 { 1 } else { -1 };
            use std::ops::Deref;
            let friction = *friction.deref();
            let new_vel = sign * friction;
            vel.y += if new_vel > -friction && new_vel < friction {
                0
            } else {
                new_vel
            }
        }

        let mut world = world.write().unwrap();
        for (mut pos, vel, actor) in (&mut pos, &vel, &actor).join() {
            pos.x = clamp(pos.x + vel.x, pos_bounds.min.x, pos_bounds.max.x);
            pos.y = clamp(pos.y + vel.y, pos_bounds.min.y, pos_bounds.max.y);
            world.place(&actor.id, pos);
        }
    }
}
