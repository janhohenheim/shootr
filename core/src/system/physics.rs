extern crate specs;

use self::specs::{Join, WriteStorage, System};

use ::model::{Pos, Vel};

pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Vel>);

    fn run(&mut self, (mut pos, mut vel): Self::SystemData) {
        let min = Pos { x: 0, y: 0 };
        let max = Pos { x: 1000, y: 1000 };
        for (mut pos, mut vel) in (&mut pos, &mut vel).join() {
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


