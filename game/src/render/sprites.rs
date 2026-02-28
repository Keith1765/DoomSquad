use std::{f64::consts::PI, rc::Rc};

use crate::game::entities::EntityType;
use crate::game::movement::Mover;
use crate::{
    game::{Game},
    render::{
        RendererData,
        camera_view::{RenderTask, RenderTaskOrderer, RenderTaskType},
    },
};

#[derive(Clone)]
pub struct ActionSpriteSwitcher {
    pub texture_id: usize,
    pub countdown: usize,
}

#[derive(Clone)]
pub struct WalkCycleHandler {
    pub current_texture_id: usize,
    pub other_texture_id: usize,
    pub countdown: usize,
    pub countdown_full_value: usize,
}

#[derive(Clone)]
pub struct Sprite {
    pub default_texture_id: usize,
    pub height: f64,
    pub width: f64,
    pub action_sprite_switcher: Option<ActionSpriteSwitcher>,
    pub walk_cycle_handler: Option<WalkCycleHandler>,
}

impl Sprite {
    pub fn get_current_sprite_texture_id(&self) -> usize {
        if let Some(switcher) = &self.action_sprite_switcher {
            //println!("{}", switcher.countdown);
            switcher.texture_id
        } else if let Some(handler) = &self.walk_cycle_handler {
            handler.current_texture_id
        } else {
            self.default_texture_id
        }
    }

    pub fn switch_sprite_for_action(&mut self, new_texture_id: usize, time: usize) {
        let switcher = ActionSpriteSwitcher {
            texture_id: new_texture_id,
            countdown: time,
        };
        self.action_sprite_switcher = Some(switcher);
    }

    pub fn continue_or_start_walk_cycle(&mut self, entity_type: &EntityType) {
        if let Some(handler) = &mut self.walk_cycle_handler {
            if handler.countdown == 0 {
                std::mem::swap(&mut handler.current_texture_id, &mut handler.other_texture_id);
                handler.countdown = handler.countdown_full_value;
            } else {
                handler.countdown -= 1;
            }
        } else { // if we are currently not animating a walk cycle, start one
            self.walk_cycle_handler = start_walk_cycle(entity_type);
        }
    }
}

pub fn start_walk_cycle(entity_type: &EntityType) -> Option<WalkCycleHandler> {
        let (current_texture_id, other_texture_id, switch_time) = entity_type.get_walk_animation_data()?;
        Some(WalkCycleHandler {
            current_texture_id,
            other_texture_id,
            countdown: switch_time,
            countdown_full_value: switch_time,
        })
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

// creates the instruction (task collection) for an entitys or interactables sprite
pub fn task_sprite(
    game: &Game,
    sprite: &Sprite,
    mover: &Mover,
    renderer_data: &RendererData,
) -> Option<SpriteInstruction> {
    // return the leftmost x of the sprite, and all the tasks to be rendered right of that
    let angle_off_player_view =
        game.player.mover.position.angle_to(&mover.position) - game.player.mover.facing_direction; // TODO abort if sprite out of FOV
    let distance: f64 = game.player.mover.position.distance_to(&mover.position);
    let normalized_distance = distance * angle_off_player_view.cos();

    // TODO temporary, find cleaner solution ?
    //idk what u mean by cleaner, but i had to change 0.0 to 0.1, because spamming bullets was crashing game, now its fine
    if normalized_distance < 0.1 {
        return None;
    }

    let onscreen_width =
        ((sprite.width / normalized_distance) * renderer_data.render_scale_coefficient) as isize;
    let onscreen_height =
        ((sprite.height / normalized_distance) * renderer_data.render_scale_coefficient) as isize;
    let onscreen_bottom: isize = ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
        + ((mover.foot_level / normalized_distance)
        - (game.player.mover.view_level / normalized_distance)) // adjust for view hieght
        * renderer_data.render_scale_coefficient) // scale correctly
        as isize;

    let center_screen_x: isize = ((renderer_data.screen_width_as_f64 / 2.0)
        + (angle_off_player_view.tan() * renderer_data.render_scale_coefficient))
        as isize;

    let left_screen_x = center_screen_x - (onscreen_width / 2);

    let angle_in_world = game.player.mover.position.angle_to(&mover.position) - 0.5 * PI; // straight line to player +90deg
    // analogous to shading for sides
    let brightness = ((angle_in_world.cos() * 0.5 + 0.75)
        / (distance * renderer_data.distance_darkness_coefficient)
        + 0.5)
        .clamp(0.2, 1.0);

    let texture = renderer_data
        .textures
        .get(&sprite.get_current_sprite_texture_id());
    let mut tasks: Vec<RenderTaskOrderer> = Vec::with_capacity(onscreen_width.max(0) as usize);

    if let Some(texture) = texture {
        // will be used often, so makes sense to cast only once
        let onscreen_width_f64 = onscreen_width as f64;

        // will be memoized when possible for optimization
        let mut texture_column: Option<Vec<u32>> =
            Some(Vec::with_capacity(onscreen_height as usize));

        // for determining if we can reuse texture_column, initialize with value well never actualy reach
        let mut prev_texture_u: usize = usize::MAX;

        for x in left_screen_x.clamp(0, renderer_data.screen_width_as_isize)
            ..(left_screen_x + onscreen_width).clamp(0, renderer_data.screen_width_as_isize)
        {
            let texture_u = ((x - left_screen_x) as f64 * (sprite.width / onscreen_width_f64))
                as usize
                % texture.width;

            // if we've gone into a new pixel column on the texture, we need to recalculate texture_column
            if texture_u != prev_texture_u {
                texture_column = texture.get_texture_column(
                    texture_u,
                    onscreen_bottom,
                    onscreen_bottom + onscreen_height,
                    sprite.height,
                    renderer_data,
                );
                prev_texture_u = texture_u;
            }

            let task = RenderTask {
                texture_column: texture_column.clone(),
                color: 0x000000, // default color, will not be read because texture exists
                brightness,
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

    Some(SpriteInstruction {
        // .clamp() is mainly to prevent overflow into fvery high numbers when casting to usize
        sprite_left_screen_x: left_screen_x.clamp(0, renderer_data.screen_width_as_isize) as usize,
        sprite_right_screen_x: (left_screen_x + onscreen_width)
            .clamp(0, renderer_data.screen_width_as_isize) as usize,
        tasks,
    })
}
