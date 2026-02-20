use crate::{
    game::{
        entities::{Entity, EntityEvent::*},
        entity_behaviour::death_behaviour,
        map_grid::MapGrid,
    },
    render::RendererData
};

use super::map::Map;
use super::player::Player;
use crate::game::damage_calculation::damage_check;
use crate::parser::entities_parser::*;
use minifb::Window;

const DESPAWN_TIME: i32 = 300;
const MAP_GRID_CELL_SIZE: f64 = 64.0;

#[derive(Clone)]
pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub map: Map,
    pub despawn_timer: i32,
    pub map_grid: MapGrid,
    pub projectile_that_hit: Vec<usize>,
}

impl Game {
    pub fn new_test_game(renderer_data: &RendererData) -> Self {
        Self {
            player: Player::new(),
            entities: parse_entities(
                "assets/maps/geogebra_test_map_with_jump+run+entities.xml".to_string(),
                &renderer_data,
            )
            .unwrap(),
            // entities: vec![
            // generate_entities(Archer,Point { x: 200.0, y: 200.0 }, 0.0, 0.0,renderer_data ),
            // generate_entities(RedBarrel, Point { x: 240.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(RangedEnemy, Point { x: 280.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(MeleeEnemy, Point { x: 320.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(MeleeEnemy, Point { x: 300.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(MeleeEnemy, Point { x: 360.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(MeleeEnemy, Point { x: 280.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(WeakEnemy, Point { x: 260.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(SummonerEnemy, Point { x: 360.0, y: 200.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(RedBarrel, Point { x: 400.0, y: 300.0 }, 0.0, 0.0, renderer_data),
            // generate_entities(Dummy, Point { x: 400.0, y: 300.0 }, 0.0, 0.0, renderer_data),
            //     ],
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
            despawn_timer: DESPAWN_TIME,
            map_grid: MapGrid::new(MAP_GRID_CELL_SIZE),
            projectile_that_hit: Vec::new(),
        }
    }

    pub fn update(&mut self, window: &Window, renderer_data: &RendererData) {
        let mut spawns = Vec::new();

        //update player
        let event = self.player.update(window, &self.map, renderer_data);
        //add possible spawn events
        spawns.extend(event);

        //update all entites and add possible spawn events
        for entity in &mut self.entities {
            let event = entity.update(window, &self.map, &self.player.mover, renderer_data);
            spawns.extend(event);
        }

        //deal damage
        damage_check(self);

        //on death behaviour
        for entity in &mut self.entities {
            if entity.hp <= 0.0 {
                spawns.extend(death_behaviour(entity, renderer_data));
            }
        }

        //despawn everything that has 0 hp
        self.entities.retain(|entity| entity.hp > 0.0);

        //spawn new entities
        for event in spawns {
            match event {
                Spawn(entity) => self.entities.push(entity),
            }
        }
    }
}
