extern crate specs;

use self::specs::{Join, WriteStorage, ReadStorage, System};

use model::{Pos, Vel, Acc};
use util::clamp;

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        ReadStorage<'a, Acc>,
    );

    fn run(&mut self, (mut pos, mut vel, acc): Self::SystemData) {
        for (mut vel, acc) in (&mut vel, &acc).join() {
            vel.x += acc.x;
            clamp(&mut vel.x, -50, 50);
            vel.y += acc.y;
            clamp(&mut vel.y, -50, 50);
        }
        for (mut pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            clamp(&mut pos.x, 0, 1000);
            pos.y += vel.y;
            clamp(&mut pos.y, 0, 1000);
        }
    }
}
