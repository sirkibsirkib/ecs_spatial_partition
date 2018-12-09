use rayon::iter::ParallelIterator;
use simple_vector2d::Vector2;
use specs::{Entities, Entity, Join, ParJoin, ReadStorage, System, WriteStorage};

use crate::components::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    SeedableRng,
};
use std::time;

pub struct BumpSystem {
    rng: StdRng,
    x_dist: Uniform<f32>,
    y_dist: Uniform<f32>,
}
impl BumpSystem {
    pub fn new() -> Self {
        BumpSystem {
            rng: SeedableRng::from_seed([0u8; 32]),
            x_dist: Uniform::new(-3., 3.),
            y_dist: Uniform::new(-3., 3.),
        }
    }
}
impl<'a> System<'a> for BumpSystem {
    type SystemData = WriteStorage<'a, Transform>;

    fn run(&mut self, mut tra: Self::SystemData) {
        (&mut tra).join().for_each(|t| {
            let offset = Vector2(
                self.x_dist.sample(&mut self.rng),
                self.y_dist.sample(&mut self.rng),
            );
            t.0 += offset;
        });
    }
}

pub struct PhysicsSystem;
impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Transform>);

    fn run(&mut self, (mut pos, mut tra): Self::SystemData) {
        (&mut pos, &mut tra).par_join().for_each(|(p, t)| {
            p.0 += t.0;
            t.0 = Vector2(0., 0.);
        })
    }
}

#[derive(Debug)]
struct Collider1D {
    range: (f32, f32),
    ent: Entity,
}
impl std::cmp::PartialEq for Collider1D {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).unwrap() == std::cmp::Ordering::Equal
    }
}
impl std::cmp::Eq for Collider1D {}
impl std::cmp::PartialOrd for Collider1D {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for Collider1D {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.range
            .0
            .partial_cmp(&other.range.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
pub struct CollisionSystem(Vec<Collider1D>);
impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem(vec![])
    }
    fn range_for(p: &Pos, c: &Collider) -> (f32, f32) {
        use self::Collider::*;
        let pt = p.0;
        let relative = match c {
            Rectangle { w, .. } => (pt.0 - (0.5 * w), pt.0 + (0.5 * w)),
            Circle { radius } => (pt.0 - radius, pt.0 + radius),
        };
        let x_pos = (p.0).0;
        (relative.0 + x_pos, relative.1 + x_pos)
    }

    // assumes that entity 1 and entity 2 are known to overlap horizontally
    // return the difference to be applied to entity 1
    fn collision_bump(p1: &Pos, c1: &Collider, p2: &Pos, c2: &Collider) -> Option<Pt> {
        use self::Collider::*;
        match (c1, c2) {
            (Rectangle { .. }, Rectangle { .. }) => {
                // let y_offset = (p2.0).1 - (p1.0).1;
                // let min_y_dist = (h1 + h2) * 0.5;
                // if y_offset < min_y_dist {
                //     let offset = p2.0 - p1.0;
                // } else { None }
                unimplemented!()
            }
            (Circle { .. }, Rectangle { .. }) => {
                // either circle origin inside rectangle
                // OR edge touches circle
                unimplemented!()
            }
            (Circle { radius: r1 }, Circle { radius: r2 }) => {
                let offset = p1.0 - p2.0;
                let min_dist = r1 + r2;
                let dist = offset.length();
                if dist < min_dist {
                    let dist_missing = min_dist - dist;
                    Some(offset * (dist_missing / dist) * 0.5)
                } else {
                    None
                }
            }
            (Rectangle { .. }, Circle { .. }) => {
                // redundant case. swap args. recursive call
                Self::collision_bump(p2, c2, p1, c1)
            }
        }
    }
}
impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Pos>,
        WriteStorage<'a, Transform>,
    );
    fn run(&mut self, (ent, col, pos, mut tra): Self::SystemData) {
        self.0.clear();
        // O(NlogN)
        (&ent, &col, &pos, &tra).join().for_each(|(e, c, p, _t)| {
            let c1d = Collider1D {
                range: Self::range_for(p, c),
                ent: e,
            };
            match self.0.binary_search(&c1d) {
                Ok(pos) => self.0.insert(pos, c1d),
                Err(pos) => self.0.insert(pos, c1d),
            }
        });
        // let mut count = 0;
        // let mut real_count = 0;

        for (i, col1) in self.0.iter().enumerate() {
            for col2 in self.0[(i + 1)..].iter() {
                if col2.range.0 < col1.range.1 {
                    // their leftmostpt is < my rightmostpt
                    // count += 1;
                    /*
                        [col2]
                        .0  .1

                    [col1]
                    .0  .1
                    */
                    if let Some(bump_to_1) = Self::collision_bump(
                        pos.get(col1.ent).unwrap(),
                        col.get(col1.ent).unwrap(),
                        pos.get(col2.ent).unwrap(),
                        col.get(col2.ent).unwrap(),
                    ) {
                        // real_count += 1;

                        let t1 = tra.get_mut(col1.ent).unwrap();
                        t1.0 += bump_to_1;
                        let t2 = tra.get_mut(col2.ent).unwrap();
                        t2.0 -= bump_to_1;
                    }
                } else {
                    break; // no more collisions for col1
                }
            }
        }
        // println!(
        //     "count {:#3} / {:#3} ({:#3.0}%). took {:?}",
        //     real_count,
        //     count,
        //     (real_count as f32 / count as f32) * 100.0,
        //     start.elapsed()
        // );
    }
}

pub struct AiSystem;
impl<'a> System<'a> for AiSystem {
    type SystemData = ReadStorage<'a, Pos>;

    fn run(&mut self, _pos: Self::SystemData) {}
}
