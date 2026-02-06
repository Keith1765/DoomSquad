use std::rc::Rc;

use crate::{
    game::{entities::Entity, map::Point, movement::Mover},
    render::{RendererData, sprites::Sprite},
};

use super::map::Map;
use super::player::Player;
use minifb::{Key, Window};

pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub map: Map,
}

impl Game {
    pub fn new_test(renderer_data: &RendererData) -> Self {
        
        Self {
            player: Player::new(),
            entities: vec![Entity::new(Point { x: 230.0, y: 210.0 }, 0.0, 20.0, 0.0, 1, renderer_data).unwrap()],
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.player.update(window, &self.map);

        for e in &mut self.entities {
            e.update(window, &self.map, &self.player.mover);
        }
    }
}
