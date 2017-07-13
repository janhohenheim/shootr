extern crate specs;

use self::specs::{Join, WriteStorage, ReadStorage, System};

use model::{Pos, Vel, Acc, BallAiInput};
use util::clamp;

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        ReadStorage<'a, Acc>,
        ReadStorage<'a, BallAiInput>,
    );

    fn run(&mut self, (mut pos, mut vel, acc, ball_ai): Self::SystemData) {
        let min = Pos { x: 0, y: 0 };
        let max = Pos { x: 1000, y: 1000 };
        for (mut pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            clamp(&mut pos.x, 0, 1000);
            pos.y += vel.y;
            clamp(&mut pos.y, 0, 1000);
        }
        for (mut vel, acc) in (&mut vel, &acc).join() {
            vel.x += acc.x;
            clamp(&mut vel.x, -15, 15);
            vel.y += acc.y;
            clamp(&mut vel.y, -15, 15);
        }
        for (mut pos, mut vel, _) in (&mut pos, &mut vel, &ball_ai).join() {
            handle_movement(&mut pos, &mut vel, &min, &max);
        }
    }
}

fn handle_movement(pos: &mut Pos, vel: &mut Vel, min: &Pos, max: &Pos) {
    pos.x += vel.x;
    pos.y += vel.y;
    if pos.x > max.x {
        vel.x = -vel.x;
        pos.x = max.x;
    }
    if pos.y > max.y {
        vel.y = -vel.y;
        pos.y = max.y;
    }
    if pos.x < min.x {
        vel.x = -vel.x;
        pos.x = min.x;
    }
    if pos.y < min.y {
        vel.y = -vel.y;
        pos.y = min.y;
    }
}
