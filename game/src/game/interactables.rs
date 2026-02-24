

use winit_input_helper::WinitInputHelper;

use crate::parser::entities_parser::{map_enemy_type, parse_entities};
use crate::{
    game::{self, generate_entities::*, map::Point, movement::Mover},
    parser::map_parser::parse_map,
    render::{RendererData, sprites::Sprite},
};
use std::{
    fmt,
    fs::{self},
    path::Path,
};

#[derive(Clone, PartialEq, Eq)]
pub enum InteractableType {
    Button(ButtonType),
    Elevator,
}
#[derive(Clone, PartialEq, Eq)]
pub enum ButtonType {
    Map,
    Spawner,
    Heal,
}

impl fmt::Display for InteractableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            InteractableType::Button(button_type) => match button_type {
                ButtonType::Map => "Map Button",
                ButtonType::Spawner => "Spawner Button",
                ButtonType::Heal => "Heal Button",
            },
            InteractableType::Elevator => "Elevator",
        };

        write!(f, "{}", text)
    }
}
#[derive(Clone)]
pub struct Interactable {
    pub mover: Mover,
    pub sprite: Sprite,
    pub player_in_range: bool,
    pub last_player_state: bool,
    pub interactable_type: InteractableType,
    pub float_1: f64,
}

impl Interactable {
    pub fn new(
        interactable_type: InteractableType,
        position: Point,
        start_floor_level: f64,
        collision_height: f64,
        float_1: f64,
        sprite_texture_id: usize,
        renderer_data: &RendererData,
    ) -> Option<Self> {
        let interactable = Interactable {
            mover: Mover {
                position,
                floor_level: start_floor_level,
                foot_level: start_floor_level,
                view_level: start_floor_level,
                height: collision_height,
                facing_direction: 0.0,
            },
            sprite: Sprite {
                default_texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
                action_sprite_switcher: None,
                walk_cycle_handler: None,
            },
            interactable_type,
            player_in_range: false,
            last_player_state: false,
            float_1: float_1,
        };
        Some(interactable)
    }
    pub fn update(
        &mut self,
        input: &WinitInputHelper,
        _renderer_data: &RendererData,
        game_state: &mut game::Game,
    ) {
        let entity_type = self.interactable_type.clone();
        match entity_type {
            InteractableType::Button(button_type) => {
                self.button_behaviour(input, _renderer_data, &button_type, game_state);
            }
            InteractableType::Elevator => {
                self.elevator_behaviour(input, _renderer_data, game_state);
            }
            _ => {}
        }
    }
    fn button_behaviour(
        &mut self,
        _input: &WinitInputHelper,
        _renderer_data: &RendererData,
        button_type: &ButtonType,
        game_state: &mut game::Game,
    ) {
        if self.player_in_range //checking if player is in range and pressing interact
            && game_state.player.interacting //checking if player pressed F for interact
            && (game_state.player.mover.foot_level - self.mover.foot_level).abs() < 5.0 //checking if player is on the same hight level 
            {
            match button_type {
                ButtonType::Map => {
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
                    let index = self.float_1 as usize;
                    if let Some(entry) = entries.get(index) {
                        let path = entry.path();
                        let map = parse_map(path.to_str().unwrap().to_string());
                        let entitties = parse_entities(path.to_str().unwrap().to_string(), _renderer_data);
                        if let Ok(map) = map {
                            game_state.map = map;
                        } else {
                            eprintln!("Error parsing map");
                            return;
                        }
                        if let Ok(entities) = entitties {
                            game_state.entities = entities;
                        } else {
                            eprintln!("Error parsing entities");
                        }
                    }
                }
                ButtonType::Spawner => {
                    println!("Spawner button pressed!");
                    let enemy_type = map_enemy_type(self.float_1 as i32);
                    println!("Spawning entity of type: {}", enemy_type);
                    let entity = generate_entities(
                        enemy_type,
                        Point {
                            x: self.mover.position.x,
                            y: self.mover.position.y,
                        },
                        self.mover.floor_level,
                        0.0,
                        _renderer_data,
                    );
                    game_state.entities.push(entity);
                }
                ButtonType::Heal => {
                    println!("Player health before: {}", game_state.player.hp);
                    let player = &mut game_state.player;
                    player.hp = player.hp + self.float_1;
                    println!("Player healed! Current HP: {}", player.hp);
                }
            }
        }
    }
    fn elevator_behaviour(
        &mut self,
        _input: &WinitInputHelper,
        _renderer_data: &RendererData,
        game_state: &mut game::Game,
    ) {
        if self.player_in_range //checking if player is in range and pressing interact
            && game_state.player.interacting //checking if player pressed F for interact
            && (game_state.player.mover.foot_level - self.mover.foot_level).abs() < 5.0 //checking if player is on the same hight level
        {
            let player = &mut game_state.player;
            player.vertical_velocity = self.float_1;
        }
    }
}
