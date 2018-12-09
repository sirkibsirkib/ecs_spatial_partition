mod systems;
use crate::systems::*;

mod components;
use crate::components::*;

use ggez::{
    conf,
    event::{self, Keycode, Mod},
    graphics::{self, DrawMode, Mesh, Point2},
    timer, Context, GameResult,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    SeedableRng,
};
use simple_vector2d::Vector2;
use specs::{world::Builder, Dispatcher, Join, World};
use std::{env, path, time};

const DESIRED_UPS: u32 = 30;

const CIRCLE_RADIUS: f32 = 10.0;

struct GameState<'a, 'b> {
    world: World,
    circle: Mesh,
    dispatcher: Dispatcher<'a, 'b>,
}
impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world = World::new();
        let _sleep_time = time::Duration::from_millis(200);

        let d_builder = specs::DispatcherBuilder::new()
            .with(BumpSystem::new(), "BumpSystem", &[])
            .with(CollisionSystem::new(), "CollisionSystem", &[])
            .with(AiSystem, "Ai", &[])
            .with_barrier()
            .with(PhysicsSystem, "PhysicsSystem", &[]);
        d_builder.print_par_seq();
        let mut dispatcher = d_builder.build();
        dispatcher.setup(&mut world.res);
        println!("using {:?} threads", dispatcher.max_threads());

        let mut rng: StdRng = SeedableRng::from_seed([0u8; 32]);
        let x_distr = Uniform::new(50., 200.);
        let y_distr = Uniform::new(50., 200.);
        for _ in 0..30 {
            world
                .create_entity()
                .with(Transform::new())
                .with(Collider::Circle {
                    radius: CIRCLE_RADIUS,
                })
                .with(Pos::new(Vector2(
                    x_distr.sample(&mut rng),
                    y_distr.sample(&mut rng),
                )))
                .build();
        }
        let circle =
            graphics::Mesh::new_circle(ctx, DrawMode::Fill, Point2::new(0., 0.), CIRCLE_RADIUS, 2.)
                .unwrap();
        Self {
            world,
            dispatcher,
            circle,
        }
    }

    pub fn update_tick(&mut self) {
        let start = time::Instant::now();
        self.dispatcher.dispatch_par(&self.world.res);
        self.world.maintain();
        println!("update took {:?}", start.elapsed());
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
        let start = time::Instant::now();
        let pos = self.world.read_storage::<Pos>();
        graphics::clear(ctx);
        for p in (&pos).join() {
            let pt = p.0;
            let param = graphics::DrawParam {
                dest: Point2::new(pt.0, pt.1),
                ..Default::default()
            };
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            let _ = graphics::draw_ex(ctx, &self.circle, param);
        }
        println!("render took {:?}", start.elapsed());
        println!("average fps {:?}", timer::get_fps(ctx));
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }
}

fn main() {
    let c = conf::Conf::default();
    let mut ctx = &mut Context::load_from_conf("super_simple", "ggez", c).unwrap();
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }
    let mut gs = GameState::new(&mut ctx);
    event::run(ctx, &mut gs).unwrap();
}
