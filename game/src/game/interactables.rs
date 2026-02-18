use crate::render::sprites::Sprite;

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

pub struct Interactable {
    pub sprite: Sprite,
    pub entity_type: InteractableType,
    pub orientation_lock: bool,
    pub cooldown: i32,
    pub hp: f64,
    pub size: f64,
}

