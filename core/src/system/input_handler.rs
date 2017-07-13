extern crate specs;

use self::specs::{Join, WriteStorage, System};

use ::model::{Vel};
pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = WriteStorage<'a, Vel>;

    fn run(&mut self, mut vel: Self::SystemData) {
        let min = Vel { x: 0, y: 0 };
        let max = Vel { x: 1000, y: 1000 };
        for mut vel in &mut vel.join() {

        }
    }
}
