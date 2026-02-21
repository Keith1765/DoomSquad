use minifb::Window;
use ron::de::Position;

use crate::{game::{map::{Map, Point}, movement::Mover, player}, render::{RendererData, sprites::Sprite}};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum InteractableType {
    Button,
    Door,
    Elevator,
    WeaponBin,
}
impl fmt::Display for InteractableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            InteractableType::Button => "Button",
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
    pub entity_type: InteractableType,
    pub player_in_range: bool,
    pub last_player_state: bool
}

// #[derive(Clone)]
// pub enum EntityEvent{
//     Spawn(Interactable),
    
// }

impl Interactable {
    pub fn new(
        entity_type: InteractableType,
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
            entity_type,
            player_in_range: false,
            last_player_state: false,
        };
        Some(interactable)
    }
    pub fn update(&mut self, window: &Window, map: &Map, _renderer_data: &RendererData) {
            match self.entity_type {
                InteractableType::Button => {
                    self.button_behaviour(window, _renderer_data);
                }
                InteractableType::Door => {
                    self.door_behaviour(window, _renderer_data);
                }
                _ => {}
            }
    }
    fn button_behaviour(&mut self, _window: &Window, _renderer_data: &RendererData) {
        if self.player_in_range != self.last_player_state {
            println!("State changed: {} at {}, {}", self.player_in_range, self.mover.position.x, self.mover.position.y);
            self.last_player_state = self.player_in_range;
        }
    }
    fn door_behaviour(&mut self, _window: &Window, _renderer_data: &RendererData) {
        //TODO
    }

}