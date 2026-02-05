use std::rc::Rc;

use minifb::{Key, Window};

use crate::game::{Game, map};
use crate::game::map::{Map, Point};
use crate::game::movement::Mover;
use crate::render::sprites::Sprite;
use crate::game::player::MOVE_SPEED;

#[derive(Clone)]
pub struct Entity {
    pub mover: Mover,
    pub sprite: Sprite,
}

impl Entity {
    // test for entity movement; simply makes it walk in a circle
    fn movement_test(self: &mut Self, map: &Map, player_position: Point) {
        self.mover.facing_direction = self.mover.position.angle_to(&player_position);
        self.mover.step(MOVE_SPEED / 4.0, 0.0, map, false);
    }

    pub fn update(self: &mut Self, window: &Window, map: &Map, player_mover: &Mover) {
        self.mover.foot_level = self.mover.floor_level;
        self.movement_test(map, player_mover.position);

        if window.is_key_down(Key::Z) {
            println!("{}",self.mover.floor_level);
        }
    }
}
