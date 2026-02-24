use crate::{game::{
        entities::{Entity, EntityEvent::*}, entity_behaviour::death_behaviour, interactables::{ButtonType, Interactable, InteractableType}, map::Point, map_grid::MapGrid
    }, render::RendererData};

use super::map::Map;
use super::player::Player;
use winit_input_helper::WinitInputHelper;
use quick_xml::events;
use crate::game::damage_calculation::damage_check;
use crate::parser::entities_parser::*;
use crate::parser::interactables_parser::*;


const DESPAWN_TIME: i32 = 300;
const MAP_GRID_CELL_SIZE: f64 = 64.0;
const INTERACTABLE_RANGE: f64 = 30.0;

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
            interactables: vec![
                Interactable::new(
                    InteractableType::Button(ButtonType::Map),
                    Point { x: 250.0, y: 5.0 },
                    0.0,
                    16.0,
                    1.0,
                    14,
                    &renderer_data,
                )
                .unwrap(),
                Interactable::new(
                    InteractableType::Button(ButtonType::Spawner),
                    Point { x: 350.0, y: 5.0 },
                    0.0,
                    16.0,
                    6.0,
                    14,
                    &renderer_data,
                )
                .unwrap(),
                Interactable::new(
                    InteractableType::Button(ButtonType::Heal),
                    Point { x: 450.0, y: 5.0 },
                    0.0,
                    16.0,
                    50.0,
                    14,
                    &renderer_data,
                )
                .unwrap(),
                Interactable::new(
                    InteractableType::Elevator,
                    Point { x: 550.0, y: 5.0 },
                    0.0,
                    16.0,
                    25.0,
                    14,
                    &renderer_data,
                ).unwrap(),
            ],
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

    pub fn update(&mut self, input: &WinitInputHelper, renderer_data: &RendererData) {
        //Interactables

        for i in 0..self.interactables.len() {
            self.player_is_in_range_of_interactable();
        }
        //update all interactables and add possible spawn events
        let mut interactables = std::mem::take(&mut self.interactables);

        for interactable in &mut interactables {
            interactable.update(input, renderer_data, self);
        }

        self.interactables = interactables;

        let mut spawns = Vec::new();
        //update player
        let event = self.player.update(input, &self.map, renderer_data);
        //add possible spawn events
        spawns.extend(event);

        //update all entites and add possible spawn events
        for entity in &mut self.entities {
            let event = entity.update(input, &self.map, &self.player.mover, renderer_data);
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

    // fn damage_check(self: &mut Self) {
    //     //DMG calc for entities

    //     //update grid
    //     self.map_grid.update(&self.entities);

    //     //let mut projectile_that_hit = Vec::new();
    //     self.projectile_that_hit.clear();

    //     //damage calc for enemies
    //     for i in 0..self.entities.len() {
    //         if (self.entities[i].entity_type != PlayerBullet)
    //             && (self.entities[i].entity_type != PlayerArrow)
    //             && (self.entities[i].entity_type != ExplodedRedBarrel)
    //         {
    //             continue;
    //         }

    //         let projectile_position = self.entities[i].mover.position;

    //         //get all entities from neighbouring cells
    //         let neighbours = self.map_grid.get_neighbours(projectile_position);

    //         for j in neighbours {
    //             let entities_immune_to_damage = matches!(
    //                 self.entities[j].entity_type,
    //                 PlayerBullet | PlayerArrow | ExplodedRedBarrel
    //             );

    //             //no self collision
    //             if i == j {
    //                 continue;
    //             }
    //             //no dmg calc for immune entities
    //             if entities_immune_to_damage {
    //                 continue;
    //             }

    //             let distance_to_bullet =
    //                 projectile_position.distance_to(&self.entities[j].mover.position);

    //             //if bullet in range of entity size
    //             if distance_to_bullet <= self.entities[j].size + self.entities[i].size {
    //                 let damage = match self.entities[i].entity_type {
    //                     PlayerBullet => BULLET_DMG,
    //                     PlayerArrow => ARROW_DMG,
    //                     ExplodedRedBarrel => RED_BARREL_DMG,
    //                     _ => 0.0,
    //                 };
    //                 //DAMAGE THAT BITCH
    //                 self.entities[j].hp -= damage;

    //                 //projectiles go brr
    //                 if (self.entities[i].entity_type != ExplodedRedBarrel) {
    //                     self.projectile_that_hit.push(i);
    //                 }
    //                 if (self.entities[i].entity_type != ExplodedRedBarrel) {
    //                     break; //cause no entity penetration
    //                 }
    //             }
    //         }
    //     }

    //     //calc damage to player
    //     let player_position = self.player.mover.position;

    //     //get all entities from neighbouring cells
    //     let neighbours = self.map_grid.get_neighbours(player_position);

    //     for j in neighbours {
    //         let distance_to_player = player_position.distance_to(&self.entities[j].mover.position);

    //         //if player size in range of entity size
    //         if distance_to_player <= self.entities[j].size + self.player.size {
    //             let damage = match self.entities[j].entity_type {
    //                 EnemyBullet => BULLET_DMG,
    //                 EnemyArrow => ARROW_DMG,
    //                 ExplodedRedBarrel => match self.entities[j].did_damage {
    //                     true => 0.0,
    //                     false => RED_BARREL_DMG,
    //                 },
    //                 MeleeEnemy => match self.entities[j].did_damage {
    //                     true => 0.0,
    //                     false => MEELE_ENEMY_DMG,
    //                 },
    //                 WeakEnemy => match self.entities[j].did_damage {
    //                     true => 0.0,
    //                     false => MEELE_ENEMY_DMG * WEAK_ENEMY_MULTIPLICATOR,
    //                 },
    //                 _ => 0.0,
    //             };
    //             //DAMAGE THAT BITCH
    //             self.player.hp -= damage;

    //             if damage > 0.0 {
    //                 match self.entities[j].entity_type {
    //                     MeleeEnemy => self.entities[j].did_damage = true,
    //                     WeakEnemy => self.entities[j].did_damage = true,
    //                     _ => {}
    //                 }
    //             }

    //             //bullet go brr
    //             if matches!(self.entities[j].entity_type, EnemyArrow | EnemyBullet) {
    //                 self.projectile_that_hit.push(j);
    //             }

    //             break; //cause no entity penetration
    //         }
    //     }

    //     //delete all bullets that hit
    //     for i in self.projectile_that_hit.clone() {
    //         self.entities[i].hp = 0.0; //o7
    //     }
    // }

    fn player_is_in_range_of_interactable(&mut self) {
        self.map_grid.update_interactables(&self.interactables);

        for interactable in &mut self.interactables {
            interactable.player_in_range = false;
        }

        let player_position = self.player.mover.position;

        let neighbours = self.map_grid.get_interactable_neighbours(player_position);

        for j in neighbours {
            let interactable_position = self.interactables[j].mover.position;
            let distance_from_player_to_interactable =
                player_position.distance_to(&interactable_position);
            if distance_from_player_to_interactable <= INTERACTABLE_RANGE {
                self.interactables[j].player_in_range = true;
            }
        }
    }
}
