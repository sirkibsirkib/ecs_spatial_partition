
use simple_vector2d::Vector2;
use specs::{
    Component, VecStorage
};
use specs_derive::Component as SComponent;

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
