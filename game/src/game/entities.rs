use std::rc::Rc;

use minifb::{Key, Window};

use crate::game::map::{Map, Point};
use crate::game::movement::Mover;
use crate::game::player::MOVE_SPEED;
use crate::game::{Game, map};
use crate::render::RendererData;
use crate::render::sprites::Sprite;

const ENTITY_DEFAULT_VIEW_HEIGHT: f64 = 15.0;
const ENTITY_MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;

#[derive(Clone)]
pub struct Entity {
    pub mover: Mover,
    pub movement_locked: bool,
    pub sprite: Sprite,
}

impl Entity {
    pub fn new(
        position: Point,
        start_floor_level: f64,
        collision_height: f64,
        facing_direction: f64,
        sprite_texture_id: usize,
        renderer_data: &RendererData,
    ) -> Option<Self> {
        let entity = Entity {
            mover: Mover {
                position, // Point { x: 230.0, y: 210.0 },
                floor_level: start_floor_level,
                foot_level: start_floor_level,
                view_level: start_floor_level + ENTITY_DEFAULT_VIEW_HEIGHT,
                height: collision_height,
                facing_direction,
            },
            movement_locked: true, // TODO change this; for now locked by default
            sprite: Sprite {
                texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
            },
        };
        Some(entity)
    }
    // test for entity movement; simply makes it walk in a circle
    fn movement_ai_test(self: &mut Self, map: &Map, player_position: Point) {
        self.mover.facing_direction = self.mover.position.angle_to(&player_position);
        self.mover.step(MOVE_SPEED, 0.0, map, false);
    }

    pub fn update(self: &mut Self, window: &Window, map: &Map, player_mover: &Mover) {
        if window.is_key_pressed(Key::L,minifb::KeyRepeat::No) {
            self.movement_locked = !self.movement_locked;
        }
        if self.movement_locked {return;}

        // smoothly make foot level catch up with floor level
        if (self.mover.foot_level - self.mover.floor_level).abs() < ENTITY_MOVEMENT_SMOOTHING_SPEED {
            self.mover.foot_level = self.mover.floor_level;
        } else if self.mover.foot_level < self.mover.floor_level {
            self.mover.foot_level += ENTITY_MOVEMENT_SMOOTHING_SPEED;
        } else {
            self.mover.foot_level -= ENTITY_MOVEMENT_SMOOTHING_SPEED;
        }
        //set view level correctly
        self.mover.view_level = self.mover.foot_level + ENTITY_DEFAULT_VIEW_HEIGHT;
        // move testwise
        self.movement_ai_test(map, player_mover.position);
    }
}
