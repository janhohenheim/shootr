extern crate specs;

use self::specs::{Fetch, Join, ReadStorage, WriteStorage, System};

use model::comp::{Acc, PlayerId};
use model::game::PlayerInputMap;
use model::client::Key;

pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = (
        Fetch<'a, PlayerInputMap>, 
        ReadStorage<'a, PlayerId>, 
        WriteStorage<'a, Acc>,
    );

    fn run(&mut self, (player_input_map, id, mut acc): Self::SystemData) {
        let player_input_map = player_input_map.read().unwrap();
        for (id, mut acc) in (&id, &mut acc).join() {
            let mut player_input = player_input_map.get(id).unwrap().write().unwrap();
            let mut key_states = &mut player_input.key_states;
            
            if let Some(state) = key_states.get_mut(&Key::ArrowUp) {
                state.fired = false;
                if state.pressed {
                    acc.y = -5
                } else if acc.y < 0 {
                    acc.y = 0
                }
            }
            if let Some(state) = key_states.get_mut(&Key::ArrowDown) {
                state.fired = false;
                if state.pressed {
                    acc.y = 5
                } else if acc.y > 0 {
                    acc.y = 0
                }
            }
        }
    }
}
