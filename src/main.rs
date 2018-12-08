use simple_vector2d::Vector2;
use smallset::SmallSet;
use specs::{
    world::Builder, Component, Entities, Entity, Join, ReadStorage, System, VecStorage, World,
    WriteStorage,
};
use specs_derive::Component as SComponent;
use std::{
    collections::{HashMap, HashSet},
    thread, time,
};

type Pt = Vector2<f32>;

use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};

#[derive(Debug, SComponent)]
#[storage(VecStorage)]
struct Pos(pub Pt);
impl Pos {
    pub const fn new(v: Pt) -> Self {
        Pos(v)
    }
}

#[derive(Debug, SComponent)]
#[storage(VecStorage)]
struct Transform(pub Pt);
impl Transform {
    pub const fn new() -> Self {
        Transform(simple_vector2d::consts::ZERO_F32)
    }
}

struct BumpSystem {
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

struct PhysicsSystem;
impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (WriteStorage<'a, Pos>, WriteStorage<'a, Transform>);

    fn run(&mut self, (mut pos, mut tra): Self::SystemData) {
        for (mut p, mut t) in (&mut pos, &mut tra).join() {
            p.0 += t.0;
            t.0 = Vector2(0., 0.);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct GridCoord(i8, i8);
impl GridCoord {
    fn closeby(self) -> impl Iterator<Item = Self> {
        ((self.0 - 1)..=(self.0 + 1))
            .map(move |x| ((self.1 - 1)..=(self.1 + 1)).map(move |y| GridCoord(x, y)))
            .flatten()
    }
}
struct CollisionSystem {
    grid: HashMap<(i8, i8), HashSet<Entity>>,
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
        for group in self.grid.values() {
            for (coord, e1) in group.iter() {
                
            }
        }
    }
}

struct AiSystem;
impl<'a> System<'a> for AiSystem {
    type SystemData = ReadStorage<'a, Pos>;

    fn run(&mut self, pos: Self::SystemData) {}
}

fn main() {
    let mut world = World::new();
    let sleep_time = time::Duration::from_millis(200);

    // define screen dims
    let (w, h) = (40, 20);
    let (x, y) = (-(w as isize) / 2, -(h as isize) / 2);

    let d_builder = specs::DispatcherBuilder::new()
        .with(BumpSystem::new(), "BumpSystem", &[])
        .with(AiSystem, "AiSystem", &[])
        .with(PhysicsSystem, "PhysicsSystem", &["BumpSystem"]);

    d_builder.print_par_seq();
    let mut dispatcher = d_builder.build();
    dispatcher.setup(&mut world.res);
    println!("using {:?} threads", dispatcher.max_threads());

    // create some entities
    let mut rng: StdRng = SeedableRng::from_seed([0u8; 32]);
    let x_distr = Uniform::new(x as f32, x as f32 + w as f32);
    let y_distr = Uniform::new(y as f32, y as f32 + h as f32);
    for _ in 0..50 {
        world
            .create_entity()
            .with(Transform::new())
            .with(Pos::new(Vector2(
                x_distr.sample(&mut rng),
                y_distr.sample(&mut rng),
            )))
            .build();
    }

    loop {
        dispatcher.dispatch(&world.res);
        world.maintain();

        thread::sleep(sleep_time);
    }
}
