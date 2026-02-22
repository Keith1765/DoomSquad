use minifb::Window;
use ron::de::Position;

use crate::{game::{map::{Map, Point}, movement::Mover, player}, render::{RendererData, sprites::Sprite}};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum InteractableType {
    Button(ButtonType),
    Door,
    Elevator,
    WeaponBin,
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
            InteractableType::Button(button_type) => {
                match button_type {
                    ButtonType::Map => "Map Button",
                    ButtonType::Spawner => "Spawner Button",
                    ButtonType::Heal => "Heal Button",
                }
            }
            InteractableType::Door => "Door",
            InteractableType::Elevator => "Elevator",
            InteractableType::WeaponBin => "WeaponBin",
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
}

// #[derive(Clone)]
// pub enum EntityEvent{
//     Spawn(Interactable),
    
// }

impl Interactable {
    pub fn new(
        interactable_type: InteractableType,
        position: Point,
        start_floor_level: f64,
        collision_height: f64, 
        sprite_texture_id: usize,
        renderer_data: &RendererData,        
    ) -> Option<Self> {
        let interactable =  Interactable{
            mover: Mover {
                position, 
                floor_level: start_floor_level,
                foot_level: start_floor_level,
                view_level: start_floor_level,
                height: collision_height,
                facing_direction: 0.0,
            },
            sprite: Sprite {
                texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
            },
            interactable_type,
            player_in_range: false,
            last_player_state: false,
        };
        Some(interactable)
    }
    pub fn update(&mut self, window: &Window, map: &Map, _renderer_data: &RendererData, player: &player::Player) {
            let entity_type = self.interactable_type.clone();
            match entity_type {
                InteractableType::Button(button_type) => {
                    self.button_behaviour(window, _renderer_data, player, &button_type);
                }
                InteractableType::Elevator => {
                    self.elevator_behaviour(window, _renderer_data);
                }
                _ => {}
            }
    }
    fn button_behaviour(&mut self, _window: &Window, _renderer_data: &RendererData, player: &player::Player, button_type: &ButtonType) {

        if self.player_in_range && player.interacting {
            match button_type {
                ButtonType::Map => {println!("Map button pressed!");},
                ButtonType::Spawner => {println!("Spawner button pressed!");},
                ButtonType::Heal => {println!("Heal button pressed!");},
            }
        }
        
    }
    fn elevator_behaviour(&mut self, _window: &Window, _renderer_data: &RendererData) {
        //TODO
    }

}