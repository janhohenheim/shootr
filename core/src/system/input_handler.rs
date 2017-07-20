extern crate specs;
use self::specs::{Fetch, Join, ReadStorage, WriteStorage, System, Entities};

use model::comp::{Acc, Player};
use model::game::{Id};
use model::client::{Key, KeyState};

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = (Fetch<'a, Arc<RwLock<HashMap<Id, Vec<KeyState>>>>>,
     WriteStorage<'a, Acc>,
     WriteStorage<'a, Player>);

    fn run(&mut self, (inputs, mut acc, mut player): Self::SystemData) {
        let inputs = inputs.read().unwrap();
        for (mut player, mut acc) in (&mut player, &mut acc).join() {
            if let Some(key_states) = inputs.get(&player.id) {
                for key_state in key_states {
                    update_player_inputs(&mut player, &key_state);
                    handle_key_state(&player, &mut acc, &key_state);
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
        },
        Key::ArrowDown => {
            if key_state.pressed {
                acc.y = 5
            } else if acc.y > 0 {
                acc.y = 0
            }
        }
        _ => {}
    }
}
