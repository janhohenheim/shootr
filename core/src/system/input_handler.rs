extern crate specs;

use self::specs::{Join, WriteStorage, System};

use model::{Acc, PlayerInput, Key};

pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = (WriteStorage<'a, PlayerInput>, WriteStorage<'a, Acc>);

    fn run(&mut self, (mut player_input, mut acc): Self::SystemData) {
        for (mut player_input, mut acc) in (&mut player_input, &mut acc).join() {
            let key_states = &mut player_input.key_states;
            if let Some(state) = key_states.get_mut(&Key::ArrowUp) {
                state.fired = false;
                if state.pressed {
                    acc.y = 5
                }
            }
            if let Some(state) = key_states.get_mut(&Key::ArrowDown) {
                state.fired = false;
                if state.pressed {
                    acc.y = -5
                }
            }
        }
    }
}
