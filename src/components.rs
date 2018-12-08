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

pub type Pt = Vector2<f32>;

#[derive(Debug, SComponent)]
#[storage(VecStorage)]
pub struct Pos(pub Pt);
impl Pos {
    pub const fn new(v: Pt) -> Self {
        Pos(v)
    }
}

#[derive(Debug, SComponent)]
#[storage(VecStorage)]
pub struct Transform(pub Pt);
impl Transform {
    pub const fn new() -> Self {
        Transform(simple_vector2d::consts::ZERO_F32)
    }
}
