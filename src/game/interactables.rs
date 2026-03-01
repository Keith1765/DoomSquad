use minifb::Window;

use crate::audio::audio_handler::Audio;
use crate::parser::entities_parser::map_enemy_type;
use crate::{
    game::{self, generate_entities::*, map::Point, movement::Mover},
    render::{RendererData, sprites::Sprite},
};
use rand::prelude::*;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum InteractableType {
    Button(ButtonType),
    Elevator,
    SlotMachine,
}

impl InteractableType {
    pub fn get_texture_id(&self) -> usize {
        match self {
            Self::Button(ButtonType::Map) => 28,
            Self::Button(ButtonType::Spawner) => 29,
            Self::Button(ButtonType::Heal) => 27,
            Self::Elevator => 26,
            Self::SlotMachine => 30,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ButtonType {
    Map,
    Spawner,
    Heal,
}

pub enum InteractableEvent {
    SpawnMap(usize),
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
            InteractableType::SlotMachine => "SlotMaschine",
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
    pub parameter_1: f64,
    pub parameter_2: f64,
    pub not_used: bool,
}

impl Interactable {
    pub fn new(
        interactable_type: InteractableType,
        position: Point,
        start_floor_level: f64,
        parameter_1: f64,
        parameter_2: f64,
        renderer_data: &RendererData,
    ) -> Option<Self> {
        let interactable = Interactable {
            mover: Mover {
                position,
                floor_level: 0.0, //not used for interactables
                foot_level: start_floor_level,
                view_level: 0.0, ////not used for interactables
                height: 0.0,     //not really need for interactables
                facing_direction: 0.0,
            },
            sprite: Sprite {
                default_texture_id: interactable_type.get_texture_id(),
                height: renderer_data
                    .textures
                    .get(&interactable_type.get_texture_id())?
                    .height as f64,
                width: renderer_data
                    .textures
                    .get(&interactable_type.get_texture_id())?
                    .width as f64,
                action_sprite_switcher: None,
                walk_cycle_handler: None,
            },
            interactable_type,
            player_in_range: false,
            last_player_state: false,
            parameter_1,
            parameter_2,
            not_used: true,
        };
        Some(interactable)
    }
    pub fn update(
        &mut self,
        window: &Window,
        _renderer_data: &RendererData,
        game_state: &mut game::Game,
        audio: &mut Audio,
    ) -> Vec<InteractableEvent> {
        let entity_type = self.interactable_type.clone();
        let mut events: Vec<InteractableEvent> = Vec::new();
        match entity_type {
            InteractableType::Button(button_type) => {
                self.button_behaviour(
                    window,
                    _renderer_data,
                    &button_type,
                    game_state,
                    &mut events,
                    audio,
                );
            }
            InteractableType::Elevator => {
                self.jump_pad_behaviour(window, _renderer_data, game_state, audio);
            }
            InteractableType::SlotMachine => {
                self.slot_maschine_behaviour(
                    window,
                    _renderer_data,
                    game_state,
                    audio,
                    &mut events,
                );
            }
        }
        events
    }
    fn button_behaviour(
        &mut self,
        _window: &Window,
        _renderer_data: &RendererData,
        button_type: &ButtonType,
        game_state: &mut game::Game,
        events: &mut Vec<InteractableEvent>,
        audio: &mut Audio,
    ) {
        if self.player_in_range //checking if player is in range and pressing interact
            && game_state.player.interacting //checking if player pressed F for interact
            && (game_state.player.mover.foot_level - self.mover.foot_level).abs() < 5.0
            && self.not_used
        //checking if player is on the same hight level
        {
            match button_type {
                ButtonType::Map => {
                    if game_state.entities.len() <= 1 {
                        events.push(InteractableEvent::SpawnMap(self.parameter_1 as usize));
                    }
                    audio.play_sfx("button_press", 1.0);
                }
                ButtonType::Spawner => {
                    let enemy_type = map_enemy_type(self.parameter_1 as i32);
                    let entity = generate_entities(
                        enemy_type,
                        Point {
                            x: self.mover.position.x,
                            y: self.mover.position.y,
                        },
                        self.mover.floor_level,
                        0.0,
                        _renderer_data,
                        0.0,
                    );
                    audio.play_sfx("summoner", 1.0);
                    if let Some(entity) = entity {
                        game_state.entities.push(entity);
                    }
                }
                ButtonType::Heal => {
                    let player = &mut game_state.player;
                    player.hp += self.parameter_1;
                    self.not_used = false;
                    audio.play_sfx("heal", 1.0);
                }
            }
        }
    }
    fn jump_pad_behaviour(
        &mut self,
        _window: &Window,
        _renderer_data: &RendererData,
        game_state: &mut game::Game,
        audio: &mut Audio,
    ) {
        if self.player_in_range //checking if player is in range and pressing interact
            && game_state.player.interacting //checking if player pressed F for interact
            && (game_state.player.mover.foot_level - self.mover.foot_level).abs() < 5.0
        //checking if player is on the same hight level
        {
            let player = &mut game_state.player;
            player.vertical_velocity = self.parameter_1;
            audio.play_sfx("jump_pad", 0.3);
        }
    }
    fn slot_maschine_behaviour(
        &mut self,
        _window: &Window,
        _renderer_data: &RendererData,
        game_state: &mut game::Game,
        audio: &mut Audio,
        events: &mut Vec<InteractableEvent>,
    ) {
        if self.player_in_range //checking if player is in range and pressing interact
            && game_state.player.interacting //checking if player pressed F for interact
            && (game_state.player.mover.foot_level - self.mover.foot_level).abs() < 5.0
        //checking if player is on the same hight level
        {
            audio.play_sfx("slotmachine", 1.0);
            let mut rng = rand::rng();

            let roll: u8 = rng.random_range(0..100);

            match roll {
                n if n > 90 => {
                    events.push(InteractableEvent::SpawnMap(0)); //swap to easteregg (testmap)
                }
                n if n < 30 => {
                    let enemy_type = map_enemy_type(self.parameter_1 as i32);
                    let entity = generate_entities(
                        enemy_type,
                        Point {
                            x: self.mover.position.x,
                            y: self.mover.position.y,
                        },
                        self.mover.floor_level,
                        0.0,
                        _renderer_data,
                        0.0,
                    );
                    if let Some(entity) = entity {
                        game_state.entities.push(entity);
                    }
                }
                n if n >= 30 => {
                    let player = &mut game_state.player;
                    player.hp += 10.0;
                }
                _ => unreachable!(),
            }
        }
    }
}
