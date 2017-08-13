extern crate specs;
use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System};

use model::comp::{Vel, Player, Actor};
use model::game::Id;
use model::network::{Command, ClientMsg};

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

type InputMap = Arc<RwLock<HashMap<Id, Vec<ClientMsg>>>>;

pub struct InputHandler;
impl<'a> System<'a> for InputHandler {
    type SystemData = (Fetch<'a, InputMap>,
     WriteStorage<'a, Vel>,
     WriteStorage<'a, Player>,
     ReadStorage<'a, Actor>);

    fn run(&mut self, (inputs, mut acc, mut player, actor): Self::SystemData) {
        let mut inputs = inputs.write().unwrap();
        for (mut player, mut vel, actor) in (&mut player, &mut acc, &actor).join() {
            if let Some(mut key_states) = inputs.get_mut(&actor.id) {
                for key_state in key_states.drain(..) {
                    update_player_inputs(&mut player, &key_state);
                    handle_key_state(player, &mut vel, &key_state);
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

fn update_player_inputs(player: &mut Player, key_state: &ClientMsg) {
    let mut input = HashMap::new();
    if let Some(last_input) = player.inputs.last() {
        input.clone_from(last_input);
        input.insert(key_state.command.clone(), key_state.active);
    }
    player.inputs.push(input);
    player.last_input = key_state.id;
}

fn handle_key_state(_: &Player, vel: &mut Vel, key_state: &ClientMsg) {
    match key_state.command {
        Command::MoveUp => {
            if key_state.active {
                vel.y = -25
            } else if vel.y < 0 {
                vel.y = 0
            }
        }
        Command::MoveDown => {
            if key_state.active {
                vel.y = 25
            } else if vel.y > 0 {
                vel.y = 0
            }
        }
    }
}
