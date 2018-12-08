use simple_vector2d::Vector2;
use smallset::SmallSet;
use specs::{
    world::Builder, Component, Entities, Entity, Join, ReadStorage, System, VecStorage, World,
    WriteStorage, Dispatcher,
};
use specs_derive::Component as SComponent;
use std::{
    path,
    env,
    collections::{HashMap, HashSet},
    thread, time,
};
use ggez::{
    conf,
    event::{self, Keycode, Mod},
    graphics::{self, spritebatch::SpriteBatch, Color, DrawMode, Mesh, Point2},
    timer, Context, GameResult,
};

const DESIRED_UPS: u32 = 2;
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct GridCoord(i8, i8);
impl GridCoord {
    fn closeby(self) -> impl Iterator<Item = Self> {
        ((self.0 - 1)..=(self.0 + 1))
            .map(move |x| ((self.1 - 1)..=(self.1 + 1)).map(move |y| GridCoord(x, y)))
            .flatten()
    }
}
struct CollisionSystem {
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


struct AiSystem;
impl<'a> System<'a> for AiSystem {
    type SystemData = ReadStorage<'a, Pos>;

    fn run(&mut self, pos: Self::SystemData) {}
}

struct GameState<'a, 'b> {
    world: World,
    circle: Mesh,
    dispatcher: Dispatcher<'a, 'b>,
}
impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(ctx: &mut Context) -> Self {
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
        let circle = graphics::Mesh::new_circle(ctx, DrawMode::Fill, Point2::new(0., 0.), 10., 2.).unwrap();
        Self {
            world,
            dispatcher,
            circle,
        }
    }

    pub fn update_tick(&mut self) {
        println!("update!");
        self.dispatcher.dispatch(&self.world.res);
        self.world.maintain();
    }
}

impl<'a, 'b> event::EventHandler for GameState<'a, 'b> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, DESIRED_UPS) {
            self.update_tick();
        }
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Escape => ctx.quit().unwrap(),
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut pos = self.world.res.get_mut::<Pos>();
        graphics::clear(ctx);
        for p in pos {
            let pt = p.0;
            println!("drawing at {}");
            let param = graphics::DrawParam {
                dest: Point2::new(pt.0, pt.1),
                ..Default::default()
            };
            graphics::draw_ex(ctx, &self.circle, param)?;
        }
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }
}


fn main() {
    let c = conf::Conf::new();
    let mut ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }
    let mut gs = GameState::new(&mut ctx);
    event::run(ctx, &mut gs).unwrap();
}