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
use specs::{
    world::Builder, Dispatcher,  Join,
    World,
};
use std::{
    env, path, time,
};

const DESIRED_UPS: u32 = 30;

struct GameState<'a, 'b> {
    world: World,
    circle: Mesh,
    dispatcher: Dispatcher<'a, 'b>,
}
impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world = World::new();
        let _sleep_time = time::Duration::from_millis(200);

        // define screen dims
        // let (w, h) = (40, 20);
        // let (x, y) = (-(w as isize) / 2, -(h as isize) / 2);

        let d_builder = specs::DispatcherBuilder::new()
            .with(BumpSystem::new(), "BumpSystem", &[])
            .with(AiSystem, "AiSystem", &[])
            .with(CollisionSystem::new(), "CollisionSystem", &[])
            .with(PhysicsSystem, "PhysicsSystem", &["BumpSystem"]);

        d_builder.print_par_seq();
        let mut dispatcher = d_builder.build();
        dispatcher.setup(&mut world.res);
        println!("using {:?} threads", dispatcher.max_threads());

        // create some entities
        let mut rng: StdRng = SeedableRng::from_seed([0u8; 32]);
        let x_distr = Uniform::new(0., 400.);
        let y_distr = Uniform::new(0., 400.);
        for _ in 0..20 {
            world
                .create_entity()
                .with(Transform::new())
                .with(Pos::new(Vector2(
                    x_distr.sample(&mut rng),
                    y_distr.sample(&mut rng),
                )))
                .build();
        }
        let circle =
            graphics::Mesh::new_circle(ctx, DrawMode::Fill, Point2::new(0., 0.), 10., 2.).unwrap();
        Self {
            world,
            dispatcher,
            circle,
        }
    }

    pub fn update_tick(&mut self) {
        // println!("update!");
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
        let pos = self.world.read_storage::<Pos>();
        graphics::clear(ctx);
        for p in (&pos).join() {
            // (&pos).par_join().for_each(|p| {
            let pt = p.0;
            // println!("drawing at {:?}", pt);
            let param = graphics::DrawParam {
                dest: Point2::new(pt.0, pt.1),
                ..Default::default()
            };
            let _ = graphics::draw_ex(ctx, &self.circle, param);
            // Ok(())
        }
        // });
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
