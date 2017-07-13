extern crate specs;

use self::specs::{Join, WriteStorage, ReadStorage, System};

use model::{Pos, Vel, Bounciness};

pub struct Bounce;
impl<'a> System<'a> for Bounce {
    type SystemData = (
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        ReadStorage<'a, Bounciness>,
    );

    fn run(&mut self, (mut pos, mut vel, bounciness): Self::SystemData) {
        let min = Pos { x: 0, y: 0 };
        let max = Pos { x: 1000, y: 1000 };
        for (mut pos, mut vel, _) in (&mut pos, &mut vel, &bounciness).join() {
            handle_movement(&mut pos, &mut vel, &min, &max);
        }
    }
}

fn handle_movement(pos: &mut Pos, vel: &mut Vel, min: &Pos, max: &Pos) {
    let next_x = pos.x + vel.x;
    let next_y = pos.y + vel.y;
    if next_x > max.x {
        vel.x = -vel.x;
        pos.x = max.x;
    }
    if next_y > max.y {
        vel.y = -vel.y;
        pos.y = max.y;
    }
    if next_x < min.x {
        vel.x = -vel.x;
        pos.x = min.x;
    }
    if next_y < min.y {
        vel.y = -vel.y;
        pos.y = min.y;
    }
}
