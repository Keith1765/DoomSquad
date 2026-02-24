use std::{fs, path::Path, usize};

use crate::{game::{
        entities::{Entity, EntityEvent::*}, entity_behaviour::death_behaviour, gamestate, interactables::{ButtonType, Interactable, InteractableType}, map::Point, map_grid::{self, MapGrid}
    }, parser::map_parser::parse_map, render::RendererData};

use super::map::Map;
use super::player::Player;
use crate::game::interactables::InteractableEvent::*;
use crate::game::damage_calculation::damage_check;
use minifb::Window;
use crate::parser::entities_parser::*;
use crate::parser::interactables_parser::*;


const DESPAWN_TIME: i32 = 300;
const MAP_GRID_CELL_SIZE: f64 = 64.0;
pub const INTERACTABLE_RANGE: f64 = 30.0;

#[derive(Clone)]
pub struct Game {
    pub player: Player,
    pub entities: Vec<Entity>,
    pub interactables: Vec<Interactable>,
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
                "assets/maps/ggb/geogebra_test_map_with_jump+run+entities.ggb".to_string(),
                &renderer_data,
            )
            .unwrap(),
            interactables : parse_interactables("assets/maps/ggb/geogebra_test_map_with_jump+run+entities.ggb".to_string(), &renderer_data).unwrap(),
            // interactables: vec![
            //     Interactable::new(
            //         InteractableType::Button(ButtonType::Map),
            //         Point { x: 250.0, y: 5.0 },
            //         0.0,
            //         1.0,
            //         0.0,
            //         14,
            //         &renderer_data,
            //     )
            //     .unwrap(),
            //     Interactable::new(
            //         InteractableType::Button(ButtonType::Spawner),
            //         Point { x: 350.0, y: 5.0 },
            //         0.0,
            //         6.0,
            //         0.0,
            //         14,
            //         &renderer_data,
            //     )
            //     .unwrap(),
            //     Interactable::new(
            //         InteractableType::Button(ButtonType::Heal),
            //         Point { x: 450.0, y: 5.0 }, //pos
            //         50.0, //floor_lvl
            //         50.0, //parameter_1
            //         0.0, //parameter_2
            //         14, //texture
            //         &renderer_data, //render_data
            //     )
            //     .unwrap(),
            //     Interactable::new(
            //         InteractableType::Elevator,
            //         Point { x: 550.0, y: 5.0 },
            //         0.0,
            //         25.0,
            //         0.0,//parameter_2
            //         14,
            //         &renderer_data,
            //     ).unwrap(),
            // ],
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
        //update all interactables and add possible spawn events
        let mut interactables = std::mem::take(&mut self.interactables);
        let mut interactables_spawns = Vec::new();

        for interactable in &mut interactables {
            let event = interactable.update(window, renderer_data, self );
            interactables_spawns.extend(event);
        }

        self.interactables = interactables;

        let mut entity_spawns = Vec::new();
        //update player
        let event = self.player.update(window, &self.map, renderer_data);
        //add possible spawn events
        entity_spawns.extend(event);

        //update all entites and add possible spawn events
        for entity in &mut self.entities {
            let event = entity.update(window, &self.map, &self.player.mover, renderer_data);
            entity_spawns.extend(event);
        }

        //deal damage
        damage_check(self);

        //on death behaviour
        for entity in &mut self.entities {
            if entity.hp <= 0.0 {
                entity_spawns.extend(death_behaviour(entity, renderer_data));
            }
        }

        //despawn everything that has 0 hp
        self.entities.retain(|entity| entity.hp > 0.0);

        //spawn new entities
        for event in entity_spawns {
            match event {
                Spawn(entity) => self.entities.push(entity),
            }
        }
        //spawn new map
        for interactables_event in interactables_spawns {
            match interactables_event {
                SpawnMap(index) => self.map_swap(renderer_data, index),
            }
        }
    }
    pub fn map_swap(self: &mut Self, renderer_data: &RendererData, map_index: usize){
        let path = Path::new("assets/maps/ggb");
                    let entries_result = fs::read_dir(path);
                    let mut entries: Vec<_> = match entries_result {
                        Ok(read_dir) => read_dir.filter_map(Result::ok).collect(),
                        Err(e) => {
                            eprintln!("Error when reading directory: {}", e);
                            return;
                        }
                    };
                    entries.sort_by_key(|e| e.path());
                    if let Some(entry) = entries.get(map_index) {
                        let path = entry.path();
                        let map = parse_map(path.to_str().unwrap().to_string());
                        let entitties = parse_entities(path.to_str().unwrap().to_string(), renderer_data);
                        let interactables=  parse_interactables(path.to_str().unwrap().to_string(), renderer_data);

                        if let Ok(map) = map {
                            self.map = map;
                        } else {
                            eprintln!("Error parsing map");
                        }
                        if let Ok(entities) = entitties {
                            self.entities = entities;
                        } else {
                            eprintln!("Error parsing entities");
                        }
                        if let Ok(interactables) = interactables {
                            self.interactables = interactables;
                        } else {
                            eprintln!("Error parsing interactables");
                            return;
                        }
                    }
    }
}
