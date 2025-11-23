use super::player::Player;
use super::map::{Map};
use minifb::Window;

pub struct Game{
    pub player: Player,
    pub map: Map,
}

impl Game {
    pub fn new() -> Self {
        Self { 
            player: Player::new(), 
            map: Map::new(),
        }
    }

    pub fn update (&mut self, window: &Window) {
        self.player.update(window, &self.map);
    }
}