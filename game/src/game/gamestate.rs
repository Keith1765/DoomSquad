use std::rc::Rc;

use crate::{
    game::{entities::{Entity, EntityEvent::{self, *}, EntityType::*},  map::Point, movement::Mover},
    render::{RendererData, sprites::Sprite},
};

use super::map::Map;
use super::player::Player;
use minifb::{Key, Window};
use quick_xml::events;

pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub map: Map,
}

impl Game {
    pub fn new_test_game(renderer_data: &RendererData) -> Self {
        
        Self {
            player: Player::new(),
            entities: vec![Entity::new(Point { x: 230.0, y: 210.0 }, 100.0, 20.0, 0.0, 3, renderer_data, RangedEnemy).unwrap()],
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
        }
    }

    pub fn update(&mut self, window: &Window, renderer_data: &RendererData) {
        self.player.update(window, &self.map);

        let mut spawns = Vec::new();

        for e in &mut self.entities {
            let event = e.update(window, &self.map, &self.player.mover, renderer_data);
            spawns.extend(event);
        }

        for event in spawns{
            match event {
                Spawn(entity) => self.entities.push(entity),
                
            }
        }
    }
}
