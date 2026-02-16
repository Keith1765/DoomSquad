use std::rc::Rc;

use crate::{
    game::{entities::{self, BULLET_DMG,ARROW_DMG, ENEMY_SIZE, Entity, EntityEvent::{self, *}, EntityType::*}, generate_entities::generate_entities, map::Point, map_grid::MapGrid, movement::Mover},
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
            entities: vec![
                generate_entities(Archer,Point { x: 200.0, y: 200.0 }, 500.0, 0.0,renderer_data ),
                // generate_entities(RedBarrel, Point { x: 240.0, y: 200.0 }, 500.0, 0.0, renderer_data),
                // generate_entities(RangedEnemy, Point { x: 280.0, y: 200.0 }, 500.0, 0.0, renderer_data),
                // generate_entities(MeleeEnemy, Point { x: 320.0, y: 200.0 }, 500.0, 0.0, renderer_data),
                // generate_entities(SummonerEnemy, Point { x: 360.0, y: 200.0 }, 500.0, 0.0, renderer_data),
                // generate_entities(RedBarrel, Point { x: 400.0, y: 300.0 }, 500.0, 0.0, renderer_data),
                ],
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
            if (self.entities[i].entity_type != PlayerBullet)&&( self.entities[i].entity_type !=PlayerArrow ){
                continue;
            }

            let bullet_position = self.entities[i].mover.position;

            //get all entities from neighbouring cells
            let neighbours = self.map_grid.get_neighbours(bullet_position);

            for j in neighbours {
                //no self collision
                if i == j {continue;}
                //no bullet on bullet collision
                if self.entities[j].entity_type == PlayerBullet {continue;}

                let distance_to_bullet = bullet_position.distance_to(&self.entities[j].mover.position);

                //if bullet in range of entity size
                if distance_to_bullet <= self.entities[j].size {
                    
                    let damage = match self.entities[i].entity_type {
                        PlayerBullet => BULLET_DMG,
                        PlayerArrow => ARROW_DMG,
                        _ => 0.0,
                    };
                    //DAMAGE THAT BITCH
                    self.entities[j].hp -= damage;

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
