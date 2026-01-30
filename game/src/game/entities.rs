use crate::game::map::Point;
use crate::game::movement::Mover;
use crate::render::sprites::Sprite;

pub struct Entity {
    pub mover: Mover,
    pub sprite: Sprite,
}
