use std::rc::Rc;

use crate::game::{Game, entities::Entity};


pub struct Sprite {
    color: u32, // TODO replace with texture
    pub height: f64,
    pub width: f64,
}

struct SpriteSlice {
    sprite: Rc<Sprite>,
    proportion: f64,
    distance: f64,
}

fn slice_sprite(game: &Game, entity: &Entity) /* -> (usize, Vec<SpriteSlice>) */ { // return the leftmost x of the sprite, and all the slices to be rendered right of that
    let distance: f64 = game.player.position.distance_to(&entity.position);
}