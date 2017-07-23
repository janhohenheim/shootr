extern crate specs;
use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System};

use model::comp::{Acc, Player, WorldId};
use model::game::Id;
use model::client::{Key, KeyState};

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

type InputMap = Arc<RwLock<HashMap<Id, Vec<KeyState>>>>;

pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = (
        Fetch<'a, InputMap>,
        WriteStorage<'a, Acc>,
        WriteStorage<'a, Player>,
        ReadStorage<'a, WorldId>,
    );

    fn run(&mut self, (inputs, mut acc, mut player, world_id): Self::SystemData) {
        let mut inputs = inputs.write().unwrap();
        for (mut player, mut acc, world_id) in (&mut player, &mut acc, &world_id).join() {
            if let Some(mut key_states) = inputs.get_mut(&world_id) {
                for key_state in key_states.drain(..) {
                    update_player_inputs(&mut player, &key_state);
                    handle_key_state(&player, &mut acc, &key_state);
                }
                let bufferlen = 10;
                let len = player.inputs.len();
                if len > bufferlen {
                    player.inputs.drain(0..len - bufferlen);
                }
            }
        }

    }
}

fn update_player_inputs(player: &mut Player, key_state: &KeyState) {
    let mut input = HashMap::new();
    if let Some(ref last_input) = player.inputs.last() {
        input.clone_from(last_input);
        input.insert(key_state.key.clone(), key_state.pressed);
    }
    player.inputs.push(input);
}

fn handle_key_state(_: &Player, acc: &mut Acc, key_state: &KeyState) {
    match key_state.key {
        Key::ArrowUp => {
            if key_state.pressed {
                acc.y = -5
            } else if acc.y < 0 {
                acc.y = 0
            }
        }
        Key::ArrowDown => {
            if key_state.pressed {
                acc.y = 5
            } else if acc.y > 0 {
                acc.y = 0
            }
        }
    }
}
