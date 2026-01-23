use crate::game::map::Point;
use crate::render::sprites::Sprite;

pub struct Entity {
    pub position: Point,
    pub vertical_position: f64,
    pub facing_angle: f64,
    pub sprite: Sprite,
}