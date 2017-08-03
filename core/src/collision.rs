use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use model::game::Vector;
use model::comp::Pos;

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
    entities: HashMap<Id, Bounds>,
    grid: HashMap<SpatialHash, Bucket<Id>>,
}

impl<Id> World<Id>
where
    Id: Hash + PartialEq + Eq + Clone,
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
    pub fn add(&mut self, id: Id, bounds: Bounds) {
        assert!(
            bounds.x + bounds.width / 2 > 0 && bounds.y + bounds.height / 2 > 0 &&
                bounds.x - bounds.width / 2 < self.width &&
                bounds.y - bounds.height / 2 < self.height
        );
        let spatial_hash = self.hash_bounds(&bounds);
        let old = self.entities.insert(id.clone(), bounds);
        assert!(
            old.is_none(),
            "Failed to add new entity: Id already registered"
        );
        self.grid
            .entry(spatial_hash)
            .or_insert_with(Bucket::new)
            .push(id);
    }

    pub fn place(&mut self, id: &Id, pos: &Pos) {
        let mut bounds = self.entities
            .get(id)
            .expect("Failed to place entity: Id doesn't exist")
            .clone();
        let old_spatial_hash = self.hash_bounds(&bounds);

        // Create new bounds
        bounds.x = pos.x;
        bounds.y = pos.y;
        assert!(
            bounds.x + bounds.width / 2 > 0 && bounds.y + bounds.height / 2 > 0 &&
                bounds.x - bounds.width / 2 < self.width &&
                bounds.y - bounds.height / 2 < self.height
        );

        let new_spatial_hash = self.hash_bounds(&bounds);
        self.entities.insert(id.clone(), bounds);

        if old_spatial_hash != new_spatial_hash {
            let id = {
                let mut bucket = self.grid.get_mut(&old_spatial_hash).expect(
                    "Failed to place entity: Didn't find bucket with existing spatial hash",
                );
                let pos = bucket.iter().position(|x| *x == *id).expect(
                    "Failed to place entity: Didn't find id in bucket",
                );
                bucket.remove(pos)
            };
            self.grid
                .entry(new_spatial_hash)
                .or_insert_with(Bucket::new)
                .push(id);
        }
    }

    pub fn remove(&mut self, id: &Id) -> Option<Bounds> {
        match self.entities.remove(id) {
            Some(bounds) => {
                let spatial_hash = self.hash_bounds(&bounds);
                let bucket = self.grid.get_mut(&spatial_hash).expect(
                    "Removed id from entity list but didn't find its spatial hash in grid",
                );
                let pos = bucket.iter().position(|x| *x == *id).expect(
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
                let bounds = &self.entities[id];
                for other_id in own_bucket {
                    if *id == *other_id || already_handled.contains(&(id, other_id)) {
                        continue;
                    }
                    let other_bounds = &self.entities[other_id];
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
                    let bounds = &self.entities[id];
                    for neighbor_id in neighbor_bucket {
                        let neighbor_bounds = &self.entities[neighbor_id];
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

    pub fn query_intersects_id<T>(&self, id: &Id, mut cb: T)
    where
        T: FnMut(CollisionObject<Id>),
    {
        let bounds = self.entities.get(id).expect(
            "Failed to query for id: Id not registered",
        );
        self.query_other(bounds, |other| if *id != *other.id &&
            bounds.intersects(other.bounds)
        {
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
                let bounds = &self.entities[id];
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

#[cfg(test)]
mod test {
    use super::*;

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

    #[derive(Clone, Debug)]
    pub struct CollisionObjectClone<Id> {
        pub id: Id,
        pub bounds: Bounds,
    }

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
    fn add() {
        let mut world = World::new(1000, 1000);
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(1, bounds);
    }

    #[test]
    #[should_panic]
    fn add_too_low() {
        let mut world = World::new(1000, 1000);
        let bounds = Bounds {
            x: -10,
            y: -10,
            width: 1,
            height: 1,
        };
        world.add(1, bounds);
    }

    #[test]
    #[should_panic]
    fn add_too_low_edge() {
        let mut world = World::new(1000, 1000);
        let bounds = Bounds {
            x: -10,
            y: 0,
            width: 5,
            height: 5,
        };
        world.add(1, bounds);
    }


    #[test]
    #[should_panic]
    fn add_too_high() {
        let mut world = World::new(1000, 1000);
        let bounds = Bounds {
            x: 2000,
            y: 2000,
            width: 1,
            height: 1,
        };
        world.add(1, bounds);
    }


    #[test]
    #[should_panic]
    fn add_too_high_edge() {
        let mut world = World::new(1000, 1000);
        let bounds = Bounds {
            x: 0,
            y: 1010,
            width: 1,
            height: 5,
        };
        world.add(1, bounds);
    }

    #[test]
    #[should_panic]
    fn add_existing() {
        let mut world = World::new(1000, 1000);
        let id = 1;
        let bounds_a = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id, bounds_a.clone());
        let bounds_b = Bounds {
            x: 1,
            y: 1,
            width: 10,
            height: 10,
        };
        world.add(id, bounds_b);
    }


    #[test]
    fn place() {
        let mut world = World::new(1000, 1000);
        let id = 1;
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id, bounds.clone());
        world.place(&id, &Vector { x: 30, y: 50 }.into());
    }


    #[test]
    #[should_panic]
    fn place_too_low() {
        let mut world = World::new(1000, 1000);
        let id = 1;
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id, bounds.clone());
        world.place(&id, &Vector { x: -999, y: 50 }.into());
    }


    #[test]
    #[should_panic]
    fn place_deleted() {
        let mut world = World::new(1000, 1000);
        let id = 1;
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id, bounds.clone());
        world.remove(&id);
        world.place(&id, &Vector { x: 30, y: 50 }.into());
    }


    #[test]
    #[should_panic]
    fn place_nonexistant() {
        let mut world = World::new(1000, 1000);
        let id = 1;
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id, bounds.clone());
        world.place(&(id + 1), &Vector { x: 30, y: 50 }.into());
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
        world.add(id, bounds.clone());
        world.remove(&id);
        world.add(id, bounds)
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
        world.add(1, bounds);
        let removed = world.remove(&2);
        assert!(removed.is_none())
    }

    #[test]
    fn remove_empty() {
        let mut world = World::new(1000, 1000);
        let removed = world.remove(&1);
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
        world.add(id, bounds.clone());
        let removed = world.remove(&id);
        assert_eq!(bounds, removed.unwrap());
        let removed = world.remove(&id);
        assert!(removed.is_none());
    }

    #[test]
    fn no_collisions() {
        let mut world = World::new(1000, 1000);
        world.add(
            1,
            Bounds {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        );
        world.add(
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
        world.add(id_a, bounds_a.clone());
        let id_b = 2;
        let bounds_b = Bounds {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };
        world.add(id_b, bounds_b.clone());

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
        world.add(id_a, bounds_a.clone());
        let not_containing = Bounds {
            x: 54,
            y: 60,
            width: 3,
            height: 6,
        };
        world.add(2, not_containing);
        let id_b = 3;
        let bounds_b = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id_b, bounds_b.clone());
        let id_c = 4;
        let bounds_c = Bounds {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };
        world.add(id_c, bounds_c.clone());
        let mut collisions = Vec::<(CollisionObjectClone<i32>, CollisionObjectClone<i32>)>::new();
        world.query_intersects(|a, b| collisions.push((a.into(), b.into())));
        assert_eq!(3, collisions.len());
        let a_b_pos = collisions
            .iter()
            .position(|&(ref lhs, ref rhs)| {
                (lhs.id == id_a && lhs.bounds == bounds_a && rhs.id == id_b &&
                     rhs.bounds == bounds_b) ||
                    (rhs.id == id_a && rhs.bounds == bounds_a && lhs.id == id_b &&
                         lhs.bounds == bounds_b)
            })
            .unwrap();
        let a_c_pos = collisions
            .iter()
            .position(|&(ref lhs, ref rhs)| {
                (lhs.id == id_a && lhs.bounds == bounds_a && rhs.id == id_c &&
                     rhs.bounds == bounds_c) ||
                    (rhs.id == id_a && rhs.bounds == bounds_a && lhs.id == id_c &&
                         lhs.bounds == bounds_c)
            })
            .unwrap();
        let b_c_pos = collisions
            .iter()
            .position(|&(ref lhs, ref rhs)| {
                (lhs.id == id_b && lhs.bounds == bounds_b && rhs.id == id_c &&
                     rhs.bounds == bounds_c) ||
                    (rhs.id == id_b && rhs.bounds == bounds_b && lhs.id == id_c &&
                         lhs.bounds == bounds_c)
            })
            .unwrap();
        assert!(a_b_pos != a_c_pos && a_b_pos != b_c_pos && a_c_pos != b_c_pos);
    }


    #[test]
    fn no_collisions_id() {
        let mut world = World::new(1000, 1000);
        world.add(
            1,
            Bounds {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        );
        world.add(
            2,
            Bounds {
                x: 40,
                y: 40,
                width: 10,
                height: 10,
            },
        );
        world.query_intersects_id(&1, |_| panic!());
    }

    #[test]
    fn one_collision_id() {
        let mut world = World::new(1000, 1000);
        let id_a = 1;
        let bounds_a = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id_a, bounds_a.clone());
        let id_b = 2;
        let bounds_b = Bounds {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };
        world.add(id_b, bounds_b.clone());

        let mut collisions = Vec::<CollisionObjectClone<i32>>::new();
        world.query_intersects_id(&id_a, |b| collisions.push(b.into()));

        assert_eq!(1, collisions.len());
        let b = collisions.first().unwrap();
        assert_eq!(id_b, b.id);
        assert_eq!(bounds_b, b.bounds);
    }


    #[test]
    fn multiple_collisions_id() {
        let mut world = World::new(1000, 1000);
        let id_a = 1;
        let bounds_a = Bounds {
            x: 3,
            y: 5,
            width: 10,
            height: 70,
        };
        world.add(id_a, bounds_a.clone());
        let not_containing = Bounds {
            x: 54,
            y: 60,
            width: 3,
            height: 6,
        };
        world.add(2, not_containing);
        let id_b = 3;
        let bounds_b = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id_b, bounds_b.clone());
        let id_c = 4;
        let bounds_c = Bounds {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
        };
        world.add(id_c, bounds_c.clone());
        let mut collisions = Vec::<CollisionObjectClone<i32>>::new();
        world.query_intersects_id(&id_c, |b| collisions.push(b.into()));
        assert_eq!(2, collisions.len());
        let a_c_pos = collisions
            .iter()
            .position(|a| a.id == id_a && a.bounds == bounds_a)
            .unwrap();
        let b_c_pos = collisions
            .iter()
            .position(|b| b.id == id_b && b.bounds == bounds_b)
            .unwrap();
        assert!(a_c_pos != b_c_pos);
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
        world.add(id, bounds_a.clone());
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
        world.add(id, bounds_a.clone());
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
        world.add(id_a, bounds_a.clone());
        let not_containing = Bounds {
            x: 54,
            y: 60,
            width: 3,
            height: 6,
        };
        world.add(2, not_containing);
        let id_b = 3;
        let bounds_b = Bounds {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        world.add(id_b, bounds_b.clone());
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
        world.add(id, bounds_a.clone());
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
        world.add(id, bounds_a.clone());
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
        world.add(id_a, bounds_a.clone());
        let not_containing = Bounds {
            x: 0,
            y: 0,
            width: 3,
            height: 6,
        };
        world.add(2, not_containing);
        let id_b = 3;
        let bounds_b = Bounds {
            x: 6,
            y: 6,
            width: 2,
            height: 2,
        };
        world.add(id_b, bounds_b.clone());
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
}
