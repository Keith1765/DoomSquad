use std::rc::Rc;

use crate::{
    game::{entities::{Entity, EntityEvent::{self, *}, EntityType::*},  map::Point, movement::Mover},
    render::{RendererData, sprites::Sprite},
};

use super::map::Map;
use super::player::Player;
use minifb::{Key, Window};
use quick_xml::events;

const DESPAWN_TIME: i32 = 300;

#[derive(Clone)]
pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub map: Map,
    pub despawn_timer: i32,
}

impl Game {
    pub fn new_test_game(renderer_data: &RendererData) -> Self {
        
        Self {
            player: Player::new(),
            entities: vec![Entity::new(Point { x: 230.0, y: 210.0 }, 100.0, 20.0, 0.0, 3, renderer_data, RangedEnemy).unwrap()],
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
            despawn_timer: DESPAWN_TIME,
        }
    }

    pub fn update(&mut self, window: &Window, renderer_data: &RendererData) {

        //TODO this is a despawn timer that despawns all bullets each time, because rn we dont know how old bullets are
        // if self.despawn_timer != 0 {
        //     self.despawn_timer -= 1;
        // }
        // //despawn all bullets after timer
        // if self.despawn_timer == 0 {
        //     self.despawn_timer = DESPAWN_TIME;
        //     self.entities.retain(|entity|entity.entity_type != Bullet);
        // }
        
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
