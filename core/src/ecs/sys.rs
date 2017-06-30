extern crate specs;

use self::specs::{Join, ReadStorage, WriteStorage, Fetch, System};

use super::comp::{Pos, Vel};
use super::res::TimeProgess;
use std::ops::Deref;


pub struct Physics;
impl<'a> System<'a> for Physics {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

pub struct Render;
impl<'a> System<'a> for Render {
    type SystemData = (Fetch<'a, TimeProgess>, ReadStorage<'a, Pos>);

    fn run(&mut self, data: Self::SystemData) {
        let (progress, pos) = data;
        let progress = progress.deref();
        for pos in pos.join() {
            println!("Progress: {:?}\tPos: {:?}", progress, pos);
        }
    }
}
