use std::{f64::consts::PI, rc::Rc};

use crate::{
    SCREEN_WIDTH,
    game::{Game, entities::Entity},
    render::{
        RendererData,
        camera_view::{RenderTask, RenderTaskOrderer, RenderTaskType},
    },
};

#[derive(Clone)]
pub struct Sprite {
    pub texture_id: usize,
    pub height: f64,
    pub width: f64,
}

// currently unused
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
    let angle_off_player_view = game.player.mover.position.angle_to(&entity.mover.position)
        - game.player.mover.facing_direction; // TODO abort if sprite out of FOV
    let distance: f64 = game
        .player
        .mover
        .position
        .distance_to(&entity.mover.position);
    let normalized_distance = distance * angle_off_player_view.cos();

    // TODO temporary, find cleaner solution ?
    //idk what u mean by cleaner, but i had to change 0.0 to 0.1, because spamming bullets was crashing game, now its fine
    if normalized_distance < 0.1 {
        return None;
    }

    let onscreen_width = ((entity.sprite.width / normalized_distance)
        * renderer_data.render_scale_coefficient) as isize;
    let onscreen_height = ((entity.sprite.height / normalized_distance)
        * renderer_data.render_scale_coefficient) as isize;
    let onscreen_bottom: isize = ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
        + ((entity.mover.foot_level / normalized_distance)
        - (game.player.mover.view_level / normalized_distance)) // adjust for view hieght
        * renderer_data.render_scale_coefficient) // scale correctly
        as isize;

    let center_screen_x: isize = ((renderer_data.screen_width_as_f64 / 2.0)
        + (angle_off_player_view.tan() * renderer_data.render_scale_coefficient))
        as isize;

    let left_screen_x = center_screen_x - (onscreen_width / 2);

    let angle_in_world = game.player.mover.position.angle_to(&entity.mover.position) - 0.5 * PI; // straight line to player +90deg
    // analogous to shading for sides
    let brightness = ((angle_in_world.cos() * 0.5 + 0.75)
        / (distance * renderer_data.distance_darkness_coefficient)
        + 0.5)
        .clamp(0.2, 1.0);

    let texture = renderer_data.textures.get(&entity.sprite.texture_id);
    let mut tasks: Vec<RenderTaskOrderer> = Vec::with_capacity(onscreen_width.max(0) as usize);

    if let Some(texture) = texture {
        // will be used often, so makes sense to cast only once
        let onscreen_width_f64 = onscreen_width.max(0) as f64;

        for x in left_screen_x..left_screen_x + onscreen_width {
            if x < 0 || x > SCREEN_WIDTH as isize - 1 {
                continue;
            }

            let texture_u = ((x - left_screen_x) as f64
                * (entity.sprite.width / onscreen_width_f64)) as usize
                % texture.width;
            let texture_column = texture.get_texture_column(
                texture_u,
                onscreen_bottom,
                onscreen_bottom + onscreen_height,
                entity.sprite.height,
                renderer_data,
            );

            let task = RenderTask {
                texture_column: texture_column,
                color: 0x000000, // default color, should not be read because texture exists
                brightness: brightness,
                onscreen_bottom,
                onscreen_top: onscreen_bottom + onscreen_height,
            };

            tasks.push(RenderTaskOrderer::new(
                task,
                distance,
                RenderTaskType::SpriteUnicolor,
            ));
        }
    }

    return Some(SpriteInstruction {
        // .max() ist to prevent overflow into fvery high numbers when casting to usize
        sprite_left_screen_x: left_screen_x.max(0) as usize,
        sprite_right_screen_x: (left_screen_x + onscreen_width).max(0) as usize,
        tasks,
    });
}
