use std::collections::HashMap;

pub type Id = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
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

pub struct World {
    width: i32,
    height: i32,
    entities: HashMap<Id, Bounds>,
}

impl World {
    pub fn new(width: i32, height: i32) -> Self {
        World {
            width,
            height,
            entities: HashMap::new(),
        }
    }
    pub fn insert(&mut self, id: Id, bounds: Bounds) -> Option<Bounds> {
        assert!(
            bounds.x + bounds.width / 2 > 0 && bounds.y + bounds.height / 2 > 0 &&
                bounds.x - bounds.width / 2 < self.width &&
                bounds.y - bounds.height / 2 < self.height
        );
        self.entities.insert(id, bounds)
    }
    pub fn remove(&mut self, id: Id) -> Option<Bounds> {
        self.entities.remove(&id)
    }
    pub fn query_intersects<T>(&self, bounds: &Bounds, mut cb: T)
    where
        T: FnMut(Id, &Bounds),
    {
        for (id, entity) in &self.entities {
            if entity.intersects(bounds) {
                cb(*id, entity);
            }
        }
    }
    pub fn query_contains<T>(&self, bounds: &Bounds, mut cb: T)
        where
            T: FnMut(Id, &Bounds),
    {
        for (id, entity) in &self.entities {
            if entity.contains(bounds) {
                cb(*id, entity);
            }
        }
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
    World::new(1000, 1000);
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
    world.query_intersects(&bounds_b, |_, _| panic!());
}

#[test]
fn one_collision() {
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
    world.query_intersects(
        &bounds_b,
        |id, bounds| collisions.push((id, bounds.clone())),
    );
    assert_eq!(1, collisions.len());
    let &(coll_id, ref coll_bounds) = collisions.first().unwrap();
    assert_eq!(id, coll_id);
    assert_eq!(bounds_a, *coll_bounds);
}


#[test]
fn multiple_collision() {
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
    world.query_intersects(
        &bounds_c,
        |id, bounds| collisions.push((id, bounds.clone())),
    );
    assert_eq!(2, collisions.len());
    let (coll_id, ref coll_bounds) = collisions[0];
    let first_is_a = id_a == coll_id;
    if first_is_a {
        assert_eq!(id_a, coll_id);
        assert_eq!(bounds_a, *coll_bounds);
    } else {
        assert_eq!(id_b, coll_id);
        assert_eq!(bounds_b, *coll_bounds);
    }
    let (coll_id, ref coll_bounds) = collisions[1];
    if first_is_a {
        assert_eq!(id_b, coll_id);
        assert_eq!(bounds_b, *coll_bounds);
    } else {
        assert_eq!(id_a, coll_id);
        assert_eq!(bounds_a, *coll_bounds);
    }
}

#[test]
fn no_containing() {
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
    world.query_contains(&bounds_b, |_, _| panic!());
}

#[test]
fn one_containing() {
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
    world.query_contains(
        &bounds_b,
        |id, bounds| containing.push((id, bounds.clone())),
    );
    assert_eq!(1, containing.len());
    let &(coll_id, ref coll_bounds) = containing.first().unwrap();
    assert_eq!(id, coll_id);
    assert_eq!(bounds_a, *coll_bounds);
}


#[test]
fn multiple_containing() {
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
    world.query_contains(
        &bounds_c,
        |id, bounds| containing.push((id, bounds.clone())),
    );
    assert_eq!(2, containing.len());
    let (coll_id, ref coll_bounds) = containing[0];
    let first_is_a = id_a == coll_id;
    if first_is_a {
        assert_eq!(id_a, coll_id);
        assert_eq!(bounds_a, *coll_bounds);
    } else {
        assert_eq!(id_b, coll_id);
        assert_eq!(bounds_b, *coll_bounds);
    }
    let (coll_id, ref coll_bounds) = containing[1];
    if first_is_a {
        assert_eq!(id_b, coll_id);
        assert_eq!(bounds_b, *coll_bounds);
    } else {
        assert_eq!(id_a, coll_id);
        assert_eq!(bounds_a, *coll_bounds);
    }
}

