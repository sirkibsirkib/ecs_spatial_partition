use ggez::{
    conf,
    event::{self, Keycode, Mod},
    graphics::{self, spritebatch::SpriteBatch, Color, DrawMode, Mesh, Point2},
    timer, Context, GameResult,
};
use rayon::iter::ParallelIterator;
use simple_vector2d::Vector2;
use smallset::SmallSet;
use specs::{
    world::Builder, Component, Dispatcher, Entities, Entity, Join, ParJoin, ReadStorage, System,
    VecStorage, World, WriteStorage,
};
use specs_derive::Component as SComponent;
use std::{
    collections::{HashMap, HashSet},
    env, path, thread, time,
};

use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use crate::components::*;

pub struct BumpSystem {
    rng: StdRng,
    x_dist: Uniform<f32>,
    y_dist: Uniform<f32>,
}
impl BumpSystem {
    pub fn new() -> Self {
        BumpSystem {
            rng: SeedableRng::from_seed([0u8; 32]),
            x_dist: Uniform::new(-1., 1.),
            y_dist: Uniform::new(-1., 1.),
        }
    }
}
impl<'a> System<'a> for BumpSystem {
    type SystemData = (ReadStorage<'a, Pos>, WriteStorage<'a, Transform>);

    fn run(&mut self, (pos, mut tra): Self::SystemData) {
        for (p, mut t) in (&pos, &mut tra).join() {
            let new_t = Transform(Vector2(
                self.x_dist.sample(&mut self.rng),
                self.y_dist.sample(&mut self.rng),
            ));
            *t = new_t;
        }
    }
}

pub struct PhysicsSystem;
impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Transform>);

    fn run(&mut self, (mut pos, mut tra): Self::SystemData) {
        // for (mut p, mut t) in (&mut pos, &mut tra).join() {
        (&mut pos, &mut tra).par_join().for_each(|(mut p, mut t)| {
            p.0 += t.0;
            t.0 = Vector2(0., 0.);
        })
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct GridCoord(i8, i8);
impl GridCoord {
    fn closeby(self) -> impl Iterator<Item = Self> {
        ((self.0 - 1)..=(self.0 + 1))
            .map(move |x| ((self.1 - 1)..=(self.1 + 1)).map(move |y| GridCoord(x, y)))
            .flatten()
    }
}
pub struct CollisionSystem {
    grid: HashMap<GridCoord, HashSet<Entity>>,
}
impl CollisionSystem {
    pub fn new() -> Self {
        CollisionSystem {
            grid: HashMap::new(),
        }
    }
    fn categorize(pos: &Pos) -> GridCoord {
        let v = pos.0;
        GridCoord((v.0 * 0.2) as i8, (v.1 * 0.2) as i8)
    }
}
impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Pos>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (ent, pos, mut tra): Self::SystemData) {
        self.grid.clear();
        for (e, p) in (&ent, &pos).join() {
            self.grid
                .entry(Self::categorize(p))
                .or_insert_with(HashSet::new)
                .insert(e);
        }
        for (coord1, group1) in self.grid.iter() {
            for coord2 in coord1.closeby() {
                if let Some(group2) = self.grid.get(&coord2) {
                    for (&e1, &e2) in group1.iter().zip(group2.iter()) {
                        if e1 == e2 {
                            continue;
                        }
                        if let (Some(p1), Some(p2)) = (pos.get(e1), pos.get(e2)) {
                            let diff = p1.0 - p2.0;
                            if diff.length() < 20. {
                                let offset1 = diff * 0.5;
                                if let Some(t) = tra.get_mut(e1) {
                                    t.0 += offset1;
                                }
                                if let Some(t) = tra.get_mut(e2) {
                                    t.0 -= offset1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct AiSystem;
impl<'a> System<'a> for AiSystem {
    type SystemData = ReadStorage<'a, Pos>;

    fn run(&mut self, pos: Self::SystemData) {}
}
