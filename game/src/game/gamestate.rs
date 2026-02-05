use std::rc::Rc;

use super::map::Map;
use super::player::Player;
use minifb::{Key, Window};

pub struct Game {
    pub player: Player,
    pub map: Map,
}

impl Game {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            map: Map::new_test_map().unwrap(), // TODO remove unwrap
        }
    }

    pub fn update(&mut self, window: &Window) {
        self.player.update(window, &self.map);

        let map = self.map.clone(); // ! TODO find more elegant solution
        for e in &mut self.map.entities {
            e.update(window, &map, &self.player.mover);
        }

        // self.map.entities.iter_mut().map(|e| e.update(window, &self));
        
    }
}
