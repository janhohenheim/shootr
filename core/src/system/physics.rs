extern crate specs;

use self::specs::{Join, WriteStorage, ReadStorage, System, Fetch};

use model::comp::{Pos, Vel, Acc, Bounds, Friction};
use util::clamp;

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>,
     WriteStorage<'a, Vel>,
     ReadStorage<'a, Acc>,
     ReadStorage<'a, Friction>,
     Fetch<'a, Bounds<Vel>>,
     Fetch<'a, Bounds<Pos>>);

    fn run(&mut self, (mut pos, mut vel, acc, friction, vel_bounds, pos_bounds): Self::SystemData) {
        for (mut vel, acc) in (&mut vel, &acc).join() {
            vel.x += acc.x;
            clamp(&mut vel.x, vel_bounds.min.x, vel_bounds.max.x);
            vel.y += acc.y;
            clamp(&mut vel.y, vel_bounds.min.y, vel_bounds.max.y);
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

        for (mut pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            clamp(&mut pos.x, pos_bounds.min.x, pos_bounds.max.x);
            pos.y += vel.y;
            clamp(&mut pos.y, pos_bounds.min.y, pos_bounds.max.y);
        }
    }
}
