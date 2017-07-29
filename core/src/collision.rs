use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::fmt::Debug;
use model::game::Vector;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
impl Bounds {
    pub fn intersects(&self, other: &Self) -> bool {
        ((self.x - other.x).abs() * 2 < (self.width + other.width)) &&
            ((self.y - other.y).abs() * 2 < (self.height + other.height))
    }
    pub fn contains(&self, other: &Self) -> bool {
        other.x - other.width / 2 >= self.x - self.width / 2 &&
            other.y - other.height / 2 >= self.y - self.height / 2 &&
            other.x + other.width / 2 <= self.x + self.width / 2 &&
            other.y + other.height / 2 <= self.y + self.height / 2
    }
}

#[derive(Clone, Debug)]
pub struct CollisionObject<'a, Id: 'a> {
    pub id: &'a Id,
    pub bounds: &'a Bounds,
}

type SpatialHash = Vector;
type Bucket<Id> = Vec<Id>;
pub struct World<Id> {
    width: i32,
    height: i32,
    cell_size: i32,
    entities: HashMap<Id, (Bounds, SpatialHash)>,
    grid: HashMap<SpatialHash, Bucket<Id>>,
}

impl<Id> World<Id>
where
    Id: Hash + PartialEq + Eq + Clone + Debug,
{
    pub fn new(width: i32, height: i32) -> Self {
        let mut grid = HashMap::new();
        let cell_size: i32 = 100;
        for i in 0..width / cell_size {
            for j in 0..height / cell_size {
                grid.insert(Vector { x: i, y: j }, Bucket::new());
            }
        }

        World {
            width,
            height,
            cell_size,
            entities: HashMap::new(),
            grid,
        }
    }
    pub fn insert(&mut self, id: Id, bounds: Bounds) -> Option<Bounds> {
        assert!(
            bounds.x + bounds.width / 2 > 0 && bounds.y + bounds.height / 2 > 0 &&
                bounds.x - bounds.width / 2 < self.width &&
                bounds.y - bounds.height / 2 < self.height
        );
        let spatial_hash = self.hash_bounds(&bounds);
        let old = self.entities.insert(
            id.clone(),
            (bounds, spatial_hash.clone()),
        );
        if old.is_none() {
            self.grid
                .entry(spatial_hash)
                .or_insert_with(Bucket::new)
                .push(id);
            None
        } else {
            let (old_bounds, old_spatial_hash) = old.unwrap();
            if spatial_hash != old_spatial_hash {
                {
                    let mut old_bucket = self.grid.get_mut(&old_spatial_hash).unwrap();
                    let pos = old_bucket.iter().position(|x| *x == id).unwrap();
                    old_bucket.remove(pos);
                }
                self.grid
                    .entry(spatial_hash)
                    .or_insert_with(Bucket::new)
                    .push(id);
            }
            Some(old_bounds)
        }
    }
    pub fn remove(&mut self, id: Id) -> Option<Bounds> {
        match self.entities.remove(&id) {
            Some((bounds, spatial_hash)) => {
                let bucket = self.grid.get_mut(&spatial_hash).expect(
                    "Removed id from entity list but didn't find its spatial hash in grid",
                );
                let pos = bucket.iter().position(|x| *x == id).expect(
                    "Didn't find id in bucket",
                );
                bucket.remove(pos);
                Some(bounds)
            }
            None => None,
        }
    }

    pub fn query_intersects<T>(&self, mut cb: T)
    where
        T: FnMut(CollisionObject<Id>, CollisionObject<Id>),
    {
        for (spatial_hash, bucket) in &self.grid {
            let neighbors = self.get_half_neighbors(spatial_hash);
            let own_bucket = &self.grid[spatial_hash];

            // Collisions in own bucket
            let mut already_handled = HashSet::new();
            for id in own_bucket {
                let &(ref bounds, _) = &self.entities[id];
                for other_id in own_bucket {
                    if *id == *other_id || already_handled.contains(&(id, other_id)) {
                        continue;
                    }
                    let &(ref other_bounds, _) = &self.entities[other_id];
                    if bounds.intersects(other_bounds) {
                        cb(
                            CollisionObject { id, bounds },
                            CollisionObject {
                                id: other_id,
                                bounds: other_bounds,
                            },
                        );
                    }
                    already_handled.insert((other_id, id));
                }
            }
            // Collisions in neighbors
            for neighbor_bucket in neighbors {
                for id in bucket {
                    let &(ref bounds, _) = &self.entities[id];
                    for neighbor_id in neighbor_bucket {
                        let &(ref neighbor_bounds, _) = &self.entities[neighbor_id];
                        if neighbor_bounds.intersects(bounds) {
                            cb(
                                CollisionObject { id, bounds },
                                CollisionObject {
                                    id: neighbor_id,
                                    bounds: neighbor_bounds,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn query_intersects_other<T>(&self, bounds: &Bounds, mut cb: T)
    where
        T: FnMut(CollisionObject<Id>),
    {
        self.query_other(bounds, |other| if bounds.intersects(other.bounds) {
            cb(other);
        });
    }

    pub fn query_contains_other<T>(&self, bounds: &Bounds, mut cb: T)
    where
        T: FnMut(CollisionObject<Id>),
    {
        self.query_other(bounds, |other| if other.bounds.contains(bounds) {
            cb(other);
        });
    }

    fn query_other<T>(&self, bounds: &Bounds, mut cb: T)
    where
        T: FnMut(CollisionObject<Id>),
    {
        let spatial_hash = self.hash_bounds(bounds);
        let mut neighbors = self.get_all_neighbors(&spatial_hash);
        let own_bucket = &self.grid[&spatial_hash];
        neighbors.push(own_bucket);
        for bucket in neighbors {
            for id in bucket {
                let &(ref bounds, _) = &self.entities[id];
                cb(CollisionObject { id, bounds })
            }
        }
    }

    fn hash_bounds(&self, bounds: &Bounds) -> SpatialHash {
        SpatialHash {
            x: bounds.x / self.cell_size,
            y: bounds.y / self.cell_size,
        }
    }

    fn get_half_neighbors(&self, spatial_hash: &SpatialHash) -> Vec<&Bucket<Id>> {
        let x = spatial_hash.x;
        let y = spatial_hash.y;

        let mut neighbors = Vec::new();

        if let Some(up) = self.grid.get(&SpatialHash { x, y: y - 1 }) {
            neighbors.push(up);
        }
        if let Some(upper_left) = self.grid.get(&SpatialHash { x: x - 1, y: y - 1 }) {
            neighbors.push(upper_left);
        }
        if let Some(left) = self.grid.get(&SpatialHash { x: x - 1, y }) {
            neighbors.push(left);
        }
        if let Some(lower_left) = self.grid.get(&SpatialHash { x: x - 1, y: y + 1 }) {
            neighbors.push(lower_left);
        }
        neighbors
    }

    fn get_all_neighbors(&self, spatial_hash: &SpatialHash) -> Vec<&Bucket<Id>> {
        let mut neighbors = self.get_half_neighbors(spatial_hash);
        let x = spatial_hash.x;
        let y = spatial_hash.y;

        if let Some(lower) = self.grid.get(&SpatialHash { x, y: y + 1 }) {
            neighbors.push(lower);
        }
        if let Some(lower_right) = self.grid.get(&SpatialHash { x: x + 1, y: y + 1 }) {
            neighbors.push(lower_right);
        }
        if let Some(right) = self.grid.get(&SpatialHash { x: x + 1, y }) {
            neighbors.push(right);
        }
        if let Some(upper_right) = self.grid.get(&SpatialHash { x: x + 1, y: y - 1 }) {
            neighbors.push(upper_right);
        }
        neighbors
    }
}


#[test]
fn intersects() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    assert!(a.intersects(&b));
}

#[test]
fn doesnt_intersect() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 50,
        y: 50,
        width: 10,
        height: 10,
    };
    assert!(!a.intersects(&b));
}

#[test]
fn doesnt_intersect_edge() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 15,
        y: 0,
        width: 10,
        height: 10,
    };
    assert!(!a.intersects(&b));
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct CollisionObjectClone<Id> {
    pub id: Id,
    pub bounds: Bounds,
}
#[cfg(test)]
impl<'a, Id> From<CollisionObject<'a, Id>> for CollisionObjectClone<Id>
where
    Id: Clone,
{
    fn from(obj: CollisionObject<'a, Id>) -> Self {
        CollisionObjectClone {
            id: obj.id.clone(),
            bounds: obj.bounds.clone(),
        }
    }
}

#[test]
fn contains() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 100,
        height: 100,
    };
    let b = Bounds {
        x: 2,
        y: 2,
        width: 10,
        height: 10,
    };
    assert!(a.contains(&b));
}

#[test]
fn doesnt_contain() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 50,
        y: 50,
        width: 10,
        height: 10,
    };
    assert!(!a.contains(&b));
}

#[test]
fn doesnt_contain_when_intersecting() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    assert!(!a.contains(&b));
}


#[test]
fn contains_edge() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let b = Bounds {
        x: 10,
        y: 10,
        width: 0,
        height: 0,
    };
    assert!(!a.contains(&b));
}

#[test]
fn contains_self() {
    let a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    assert!(a.contains(&a));
}

#[test]
fn init() {
    World::<i32>::new(1000, 1000);
}

#[test]
fn insert_new() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let old = world.insert(1, bounds);
    assert!(old.is_none());
}

#[test]
#[should_panic]
fn insert_too_low() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: -10,
        y: -10,
        width: 1,
        height: 1,
    };
    world.insert(1, bounds);
}

#[test]
#[should_panic]
fn insert_too_low_edge() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: -10,
        y: 0,
        width: 5,
        height: 5,
    };
    world.insert(1, bounds);
}


#[test]
#[should_panic]
fn insert_too_high() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: 2000,
        y: 2000,
        width: 1,
        height: 1,
    };
    world.insert(1, bounds);
}


#[test]
#[should_panic]
fn insert_too_high_edge() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: 0,
        y: 1010,
        width: 1,
        height: 5,
    };
    world.insert(1, bounds);
}

#[test]
fn insert_existing() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    let old = world.insert(id, bounds_a.clone());
    assert!(old.is_none());
    let bounds_b = Bounds {
        x: 1,
        y: 1,
        width: 10,
        height: 10,
    };
    let old = world.insert(id, bounds_b);
    assert_eq!(bounds_a, old.unwrap())
}

#[test]
fn remove() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds.clone());
    let removed_bounds = world.remove(id);
    assert_eq!(bounds, removed_bounds.unwrap());
    let old = world.insert(id, bounds);
    assert!(old.is_none());
}

#[test]
fn remove_nonexistant() {
    let mut world = World::new(1000, 1000);
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(1, bounds);
    let removed = world.remove(2);
    assert!(removed.is_none())
}

#[test]
fn remove_empty() {
    let mut world = World::new(1000, 1000);
    let removed = world.remove(1);
    assert!(removed.is_none())
}

#[test]
fn remove_twice() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds.clone());
    let removed = world.remove(id);
    assert_eq!(bounds, removed.unwrap());
    let removed = world.remove(id);
    assert!(removed.is_none());
}

#[test]
fn no_collisions() {
    let mut world = World::new(1000, 1000);
    world.insert(
        1,
        Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        },
    );
    world.insert(
        2,
        Bounds {
            x: 40,
            y: 40,
            width: 10,
            height: 10,
        },
    );
    world.query_intersects(|_, _| panic!());
}

#[test]
fn one_collision() {
    let mut world = World::new(1000, 1000);
    let id_a = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id_a, bounds_a.clone());
    let id_b = 2;
    let bounds_b = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    world.insert(id_b, bounds_b.clone());

    let mut collisions = Vec::<(CollisionObjectClone<i32>, CollisionObjectClone<i32>)>::new();
    world.query_intersects(|a, b| collisions.push((a.into(), b.into())));

    assert_eq!(1, collisions.len());
    let &(ref a, ref b) = collisions.first().unwrap();
    assert_eq!(id_a, a.id);
    assert_eq!(id_b, b.id);
    assert_eq!(bounds_a, a.bounds);
    assert_eq!(bounds_b, b.bounds);
}


#[test]
fn multiple_collisions() {
    let mut world = World::new(1000, 1000);
    let id_a = 1;
    let bounds_a = Bounds {
        x: 3,
        y: 5,
        width: 10,
        height: 70,
    };
    world.insert(id_a, bounds_a.clone());
    let not_containing = Bounds {
        x: 54,
        y: 60,
        width: 3,
        height: 6,
    };
    world.insert(2, not_containing);
    let id_b = 3;
    let bounds_b = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id_b, bounds_b.clone());
    let id_c = 4;
    let bounds_c = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    world.insert(id_c, bounds_c.clone());
    let mut collisions = Vec::<(CollisionObjectClone<i32>, CollisionObjectClone<i32>)>::new();
    world.query_intersects(|a, b| collisions.push((a.into(), b.into())));
    assert_eq!(2, collisions.len());
    let (ref a, ref b) = collisions[0];
}


#[test]
fn no_collisions_other() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds_a.clone());
    let bounds_b = Bounds {
        x: 40,
        y: 40,
        width: 10,
        height: 10,
    };
    world.query_intersects_other(&bounds_b, |_| panic!());
}

#[test]
fn one_collision_other() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds_a.clone());
    let bounds_b = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    let mut collisions = Vec::new();
    world.query_intersects_other(&bounds_b, |obj| {
        collisions.push((*obj.id, obj.bounds.clone()))
    });
    assert_eq!(1, collisions.len());
    let &(coll_id, ref coll_bounds) = collisions.first().unwrap();
    assert_eq!(id, coll_id);
    assert_eq!(bounds_a, *coll_bounds);
}


#[test]
fn multiple_collision_other() {
    let mut world = World::new(1000, 1000);
    let id_a = 1;
    let bounds_a = Bounds {
        x: 3,
        y: 5,
        width: 10,
        height: 70,
    };
    world.insert(id_a, bounds_a.clone());
    let not_containing = Bounds {
        x: 54,
        y: 60,
        width: 3,
        height: 6,
    };
    world.insert(2, not_containing);
    let id_b = 3;
    let bounds_b = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id_b, bounds_b.clone());
    let bounds_c = Bounds {
        x: 5,
        y: 5,
        width: 10,
        height: 10,
    };
    let mut collisions = Vec::new();
    world.query_intersects_other(&bounds_c, |obj| {
        collisions.push((*obj.id, obj.bounds.clone()))
    });
    assert_eq!(2, collisions.len());
    let (coll_id, ref coll_bounds) = collisions[0];
    assert_eq!(id_a, coll_id);
    assert_eq!(bounds_a, *coll_bounds);

    let (coll_id, ref coll_bounds) = collisions[1];
    assert_eq!(id_b, coll_id);
    assert_eq!(bounds_b, *coll_bounds);

}

#[test]
fn no_containing_other() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds_a.clone());
    let bounds_b = Bounds {
        x: 40,
        y: 40,
        width: 10,
        height: 10,
    };
    world.query_contains_other(&bounds_b, |_| panic!());
}

#[test]
fn one_containing_other() {
    let mut world = World::new(1000, 1000);
    let id = 1;
    let bounds_a = Bounds {
        x: 0,
        y: 0,
        width: 10,
        height: 10,
    };
    world.insert(id, bounds_a.clone());
    let bounds_b = Bounds {
        x: 2,
        y: 2,
        width: 3,
        height: 3,
    };
    let mut containing = Vec::new();
    world.query_contains_other(&bounds_b, |obj| {
        containing.push((*obj.id, obj.bounds.clone()))
    });
    assert_eq!(1, containing.len());
    let &(coll_id, ref coll_bounds) = containing.first().unwrap();
    assert_eq!(id, coll_id);
    assert_eq!(bounds_a, *coll_bounds);
}


#[test]
fn multiple_containing_other() {
    let mut world = World::new(1000, 1000);
    let id_a = 1;
    let bounds_a = Bounds {
        x: 4,
        y: 4,
        width: 100,
        height: 100,
    };
    world.insert(id_a, bounds_a.clone());
    let not_containing = Bounds {
        x: 0,
        y: 0,
        width: 3,
        height: 6,
    };
    world.insert(2, not_containing);
    let id_b = 3;
    let bounds_b = Bounds {
        x: 6,
        y: 6,
        width: 2,
        height: 2,
    };
    world.insert(id_b, bounds_b.clone());
    let bounds_c = Bounds {
        x: 5,
        y: 5,
        width: 1,
        height: 1,
    };
    let mut containing = Vec::new();
    world.query_contains_other(&bounds_c, |obj| {
        containing.push((*obj.id, obj.bounds.clone()))
    });
    assert_eq!(2, containing.len());
    let (coll_id, ref coll_bounds) = containing[0];
    assert_eq!(id_a, coll_id);
    assert_eq!(bounds_a, *coll_bounds);

    let (coll_id, ref coll_bounds) = containing[1];
    assert_eq!(id_b, coll_id);
    assert_eq!(bounds_b, *coll_bounds);
}
