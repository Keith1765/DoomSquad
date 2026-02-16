use std::rc::Rc;

use crate::{
    game::{entities::{Entity, EntityEvent::{self, *}, EntityType::*, BULLET_DMG, ENEMY_SIZE}, map::Point, map_grid::MapGrid, movement::Mover},
    render::{RendererData, sprites::Sprite},
};

use super::map::Map;
use super::player::Player;
use minifb::{Key, Window};
use quick_xml::events;

const DESPAWN_TIME: i32 = 300;
const MAP_GRID_CELL_SIZE: f64 = 64.0;

#[derive(Clone)]
pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub map: Map,
    pub despawn_timer: i32,
    pub map_grid: MapGrid,
}

impl Game {
    pub fn new_test_game(renderer_data: &RendererData) -> Self {
        
        Self {
            player: Player::new(),
            entities: vec![Entity::new(Point { x: 230.0, y: 210.0 }, 100.0, 20.0, 0.0, 6, renderer_data, SummonerEnemy, 100.0, ENEMY_SIZE).unwrap()],
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
            despawn_timer: DESPAWN_TIME,
            map_grid: MapGrid::new(MAP_GRID_CELL_SIZE), 
        }
    }

    pub fn update(&mut self, window: &Window, renderer_data: &RendererData) {

        self.damage_check();

        //despawn everything that has 0 hp
        self.entities.retain(|entity|entity.hp > 0.0);
        
        let mut spawns = Vec::new();
        //update player
        let event = self.player.update(window, &self.map,renderer_data);
        //add possible spawn events
        spawns.extend(event);

        //update all entites and add possible spawn events
        for e in &mut self.entities {
            let event = e.update(window, &self.map, &self.player.mover, renderer_data);
            spawns.extend(event);
        }

        //spawn new entities
        for event in spawns{
            match event {
                Spawn(entity) => self.entities.push(entity),
                
            }
        }
    }

     fn damage_check (self: &mut Self) {
        //update grid
        self.map_grid.update(&self.entities);

        let mut bullets_that_hit = Vec::new();

        for i in 1..self.entities.len() {
            //currently only for bullet, but extendable
            if self.entities[i].entity_type != Bullet {
                continue;
            }

            let bullet_position = self.entities[i].mover.position;

            //get all entities from neighbouring cells
            let neighbours = self.map_grid.get_neighbours(bullet_position);

            for j in neighbours {
                //no self collision
                if i == j {continue;}
                //no bullet on bullet collision
                if self.entities[j].entity_type == Bullet {continue;}

                let distance_to_bullet = bullet_position.distance_to(&self.entities[j].mover.position);

                //if bullet in range of entity size
                if distance_to_bullet <= self.entities[j].size {
                    //DAMAGE THAT BITCH
                    self.entities[j].hp -= BULLET_DMG;

                    //bullet go brr
                    bullets_that_hit.push(i);

                    break; //cause no entity penetration
                }
            }
        }

        //delete all bullets that hit
        for i in bullets_that_hit{
            self.entities[i].hp = 0.0; //o7
        }

    }

}
