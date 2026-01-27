use std::ops::Rem;
use std::{f64::consts::PI, rc::Rc};

use crate::{
    SCREEN_WIDTH,
    game::{Game, entities::Entity},
    render::{
        RendererData,
        camera_view::{RenderTask, RenderTaskOrderer, RenderTaskType},
    },
};

pub struct Sprite {
    pub color: u32, // TODO replace with texture
    pub height: f64,
    pub width: f64,
}

struct SpriteSlice {
    sprite: Rc<Sprite>,
    proportion: f64,
    distance: f64,
}

pub struct SpriteInstruction {
    pub sprite_left_screen_x: usize,
    pub sprite_right_screen_x: usize,
    pub tasks: Vec<RenderTaskOrderer>,
}

// creates the instruction (task collection) for an entitys sprite
pub fn task_sprite(
    game: &Game,
    entity: &Entity,
    renderer_data: &RendererData,
) -> Option<SpriteInstruction> {
    // return the leftmost x of the sprite, and all the tasks to be rendered right of that
    let angle_off_player_view =
        game.player.position.angle_to(&entity.position) - game.player.view_angle; // TODO abort if sprite out of FOV
    let distance: f64 = game.player.position.distance_to(&entity.position);
    let normalized_distance = distance * angle_off_player_view.cos();

    // TODO temporary, find cleaner solution ?
    if normalized_distance < 0.0 {
        return None;
    }

    let onscreen_width = ((entity.sprite.width / normalized_distance)
        * renderer_data.render_scale_coefficient) as isize; // TODO rename this coeff
    let onscreen_height = ((entity.sprite.height / normalized_distance)
        * renderer_data.render_scale_coefficient) as isize; // TODO rename this coeff

    let bottom_onscreen: isize = ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
        + ((entity.vertical_position / normalized_distance)
        - (game.player.view_height / normalized_distance)) // adjust for view hieght
        * renderer_data.render_scale_coefficient) // scale correctly
        as isize;

    let center_screen_x: isize = ((renderer_data.screen_width_as_f64 / 2.0)
        + (angle_off_player_view.tan() * renderer_data.render_scale_coefficient))
        as isize;

    let left_screen_x = center_screen_x - (onscreen_width / 2);

    let angle_in_world = game.player.position.angle_to(&entity.position) - 0.5 * PI; // straight line to player +90deg
    // analogous to shading for sides
    let brightness = ((angle_in_world.cos() * 0.5 + 0.75)
        / (distance * renderer_data.distance_darkness_coefficient)
        + 0.5)
        .clamp(0.2, 1.0);

    let mut tasks: Vec<RenderTaskOrderer> = Vec::with_capacity(onscreen_width as usize);

    for x in left_screen_x..left_screen_x + onscreen_width {
        if x < 0 || x > SCREEN_WIDTH as isize - 1 {
            continue;
        }

        let task = RenderTask {
            texture_column: None, // TODO change this to enable textures for spirites
            color: entity.sprite.color,
            brightness: brightness,
            onscreen_bottom: bottom_onscreen,
            onscreen_top: bottom_onscreen + onscreen_height,
        };

        tasks.push(RenderTaskOrderer::new(
            task,
            distance,
            RenderTaskType::SpriteUnicolor,
        ));
    }

    return Some(SpriteInstruction {
        // .max() ist to prevent overflow into fvery high numbers when casting to usize
        sprite_left_screen_x: left_screen_x.max(0) as usize,
        sprite_right_screen_x: (left_screen_x + onscreen_width).max(0) as usize,
        tasks,
    });
}
