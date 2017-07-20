extern crate specs;

use self::specs::{Fetch, Join, WriteStorage, ReadStorage, System, Entities, EntitiesRes, Entity};

use model::comp::{Pos, Vel, Acc, PlayerId, Friction, Connect, Disconnect};

struct PlayerData {
    pos: WriteStorage<'a, Pos>,
    vel: WriteStorage<'a, vel>,
    acc: WriteStorage<'a, Acc>,
    friction: WriteStorage<'a, Friction>,
    id: WriteStorage<'a, PlayerId>,
}

pub struct Spawn;
impl<'a> System<'a> for Spawn {
    type SystemData = (
        Entities<'a>,
        PlayerData<'a>,
        WriteStorage<'a, Connect>,
        ReadStorage<'a, Disconnect>,
     );

    fn run(&mut self, (mut entities, mut player_data, mut connect, disconnect): Self::SystemData) {
        let mut to_spawn = Vec::new();
        for (entity, _) in (&*entities, &connect).join() {
            to_spawn.push(entity);
        }
        spawn(&to_spawn, &mut player_data, &mut connect)

        let mut to_despawn = Vec::new();
        for (entity, _) in (&*entities, &disconnect).join() {
            to_despawn.push(entity);
        }
        despawn(&to_despawn, &mut entitiest);
    }
}

fn spawn(to_spawn: &Vec<Entity>, data: &mut PlayerData, connect: &mut WriteStorage<'a, Connect>) {
    for entity in to_spawn {
        let id = entity.id;
        let x = if entity.id % 2 == 0 { 20 } else { 980 };
        data.acc.insert(entity, Acc::from(Vector { x: 0, y: 0 }));
        data.vel.insert(entity, Vel::from(Vector { x: 0, y: 0 }));
        data.pos.insert(entity, Pos::from(Vector { x, y: 500 }));
        data.friction.insert(entity, Friction(2));
        data.id.insert(entity, PlayerId(id));
        connect.remove(entity);
    }
}

fn despawn(to_despawn: &Vec<Entity>, entities: &mut EntitiesRes) {
     for entity in to_despawn {
         entities.delete(entity);
     }
}
