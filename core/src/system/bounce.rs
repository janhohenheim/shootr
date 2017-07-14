extern crate specs;

use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System};

use model::{Bounds, Pos, Vel, Bounciness};

pub struct Bounce;
impl<'a> System<'a> for Bounce {
    type SystemData = (
        ReadStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        ReadStorage<'a, Bounciness>,
        Fetch<'a, Bounds<Pos>>
    );

    fn run(&mut self, (pos, mut vel, bounciness, pos_bounds): Self::SystemData) {
        for (pos, mut vel, _) in (&pos, &mut vel, &bounciness).join() {
            handle_movement(pos, &mut vel, &pos_bounds);
        }
    }
}

fn handle_movement(pos: &Pos, vel: &mut Vel, bounds: &Bounds<Pos>) {
    let next_x = pos.x + vel.x;
    let next_y = pos.y + vel.y;
    if next_x > bounds.max.x || next_x < bounds.min.x {
        vel.x = -vel.x;
    } 

    if next_y > bounds.max.y || next_y < bounds.min.y {
        vel.y = -vel.y;
    } 
}
