use core::f64;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::game::Game;
use crate::render::blocks_walls::task_column;
use crate::render::crosshair::draw_crosshair;
use crate::render::player_hp_bar::draw_player_hp_bar;
// TODO LEVEL_HEIGHT and other map data into sth similar to renderer_data?
use crate::render::raycast::{MapSlice, raycast};
use crate::render::renderer_init::RendererData;
use crate::render::sprites::task_sprite;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH}; // TODO fully move this into renderer_data (currently problem because arraysize wants constant, typing)

pub type VerticalDisctance = f64;

/// types of renderer tasks: we always draw a floor or ceiling, a side of a block7wall, or a sprite
#[derive(Clone, Copy, PartialEq)]
pub enum RenderTaskType {
    Floor(VerticalDisctance), // vert dist is needed for sorting between surface tasks
    Ceiling(VerticalDisctance),
    Sprite,
    SideTexture,
}

/// a render task is an instruction for the renderer to draw something on a part of the screen; they are orederd by an orderer
/// so that only the frontmost are visible in the end
#[derive(Clone)]
pub struct RenderTask {
    pub texture_column: Option<Vec<u32>>, // texture and color will never both be used
    pub color: u32,
    pub brightness: f64,
    pub onscreen_bottom: isize,
    pub onscreen_top: isize,
}

/// a wrapper for a task to order it, as explained above
#[derive(Clone)]
pub struct RenderTaskOrderer {
    pub task: RenderTask,
    pub(crate) task_type: RenderTaskType,
    pub(crate) distance: f64,
}

impl PartialEq for RenderTaskOrderer {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for RenderTaskOrderer {} // PartialEQ already handles functionality, but must be written out; do not remove

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for RenderTaskOrderer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // surfaces should be rendered above sides when they are of equal (within floating point error) distance, to prevent flickering between the two
        if (self.distance - other.distance).abs() < 0.1 {
            match (self.task_type, other.task_type) {
                // if both are ciling or both are floor, render the one that is vertically closer above further one (prevents flickering between surface of equal distance)
                (RenderTaskType::Ceiling(s_vert_dist), RenderTaskType::Ceiling(o_vert_dist))
                | (RenderTaskType::Floor(s_vert_dist), RenderTaskType::Floor(o_vert_dist)) => {
                    s_vert_dist.partial_cmp(&o_vert_dist)
                }
                // if both are surfaces (ceilings or floors), but not of same type, order as normal
                (RenderTaskType::Ceiling(_), RenderTaskType::Floor(_))
                | (RenderTaskType::Floor(_), RenderTaskType::Ceiling(_)) => {
                    self.distance.partial_cmp(&other.distance)
                }
                // if self is a surface but not other, render self above other
                (RenderTaskType::Floor(_), _) | (RenderTaskType::Ceiling(_), _) => {
                    Some(Ordering::Less)
                }
                // if other is a surface but not self, render other above self
                (_, RenderTaskType::Floor(_)) | (_, RenderTaskType::Ceiling(_)) => {
                    Some(Ordering::Greater)
                }
                // in all other cases (if neither are surfaces), render in normal ordering
                (_, _) => self.distance.partial_cmp(&other.distance),
            }
        } else {
            // if not very close, order as one would expect, accorfding to distance
            self.distance.partial_cmp(&other.distance)
        }
    }
}

impl Ord for RenderTaskOrderer {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(ordering) = self.distance.partial_cmp(&other.distance) {
            ordering
        } else {
            Ordering::Equal // as default, should never actually happen
        }
    }
}

impl RenderTaskOrderer {
    pub fn new(task: RenderTask, distance: f64, task_type: RenderTaskType) -> Self {
        RenderTaskOrderer {
            task,
            distance,
            task_type,
        }
    }
}

// the collection of tasks for a column of the screen, with the furthest distance behind which nothing is rendered, becasue theres a wall there
pub struct ColumnTasks {
    pub tasks: BinaryHeap<RenderTaskOrderer>,
    pub wall_distance: f64,
}

// draws the whole screen into the buffer, to be drawn by library into the window later
pub fn draw_screen(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    //write grey plane as background to overwrite past frames
    for px in buffer.iter_mut() {
        *px = renderer_data.background_color;
    }
    draw_camera_view(buffer, renderer_data, game);
    //draw grid of reference points spaced each 50 pixels for debugging
    if game.player.godmode {
        draw_reference_points(buffer);
    }

    //draw playwer hp bar
    draw_player_hp_bar(buffer, renderer_data, game.player.hp);

    //draw crosshair
    draw_crosshair(
        buffer,
        game.player.vertcal_aim,
        renderer_data,
        game.player.aim_mode,
    );
}

/// draws the camera view (screen without hud and such)
fn draw_camera_view(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    // for every column of the screen, create a slice of the map with a raycast

    // stores every map slice together with the angle relative to the player with which it was cast
    // this angle will be needed for distortion correction
    let mut map_slices_and_angles: [Option<(MapSlice, f64)>; SCREEN_WIDTH] =
        [const { None }; SCREEN_WIDTH];

    #[allow(clippy::needless_range_loop)]
    for x in 0..SCREEN_WIDTH {
        let pixel_distance_from_screen_middle: f64 = x as f64 - SCREEN_WIDTH as f64 / 2.0;
        let angle_relative_to_player: f64 =
            (pixel_distance_from_screen_middle / renderer_data.render_scale_coefficient).atan();

        map_slices_and_angles[x] = Some((
            raycast(&game.map, angle_relative_to_player, &game.player),
            angle_relative_to_player,
        ));
    }

    // convert every mapslice into taskings

    // the taskings for each column of the screen
    let mut columns_tasked: [Option<ColumnTasks>; SCREEN_WIDTH] = [const { None }; SCREEN_WIDTH];

    for x in 0..SCREEN_WIDTH {
        if let Some((map_slice, angle_relative_to_player)) = &map_slices_and_angles[x] {
            columns_tasked[x] = Some(task_column(
                game,
                renderer_data,
                map_slice,
                *angle_relative_to_player,
            ));
        }
    }

    // create entity (sprite) tasks, put them into the taskings
    for entity in &game.entities {
        // get a entity tasking instruction
        if let Some(mut instruction) =
            task_sprite(game, &entity.sprite, &entity.mover, renderer_data)
        {
            // draw the columns of that instructon into the correct columns of the screen
            let sprite_width = instruction.sprite_right_screen_x - instruction.sprite_left_screen_x;
            for x in 0..sprite_width {
                if x > SCREEN_WIDTH - 1 {
                    continue;
                }

                if let Some(column_tasks) =
                    &mut columns_tasked[instruction.sprite_right_screen_x - x - 1]
                    && let Some(sprite_task) = instruction.tasks.pop()
                    && sprite_task.distance <= column_tasks.wall_distance
                {
                    column_tasks.tasks.push(sprite_task);
                }
            }
        }
    }

    // create interactable (sprite) tasks, put them into the taskings
    // practically identical to above
    for interactable in &game.interactables {
        if let Some(mut instruction) = task_sprite(
            game,
            &interactable.sprite,
            &interactable.mover,
            renderer_data,
        ) {
            let sprite_width = instruction.sprite_right_screen_x - instruction.sprite_left_screen_x;
            for x in 0..sprite_width {
                if x > SCREEN_WIDTH - 1 {
                    continue;
                }

                if let Some(cts) = &mut columns_tasked[instruction.sprite_right_screen_x - x - 1]
                    && let Some(sprite_task) = instruction.tasks.pop()
                    && sprite_task.distance <= cts.wall_distance
                {
                    cts.tasks.push(sprite_task);
                }
            }
        }
    }

    // draw all the tasks into the buffer
    for x in 0..SCREEN_WIDTH {
        if let Some(column_tasks) = &mut columns_tasked[x] {
            // create a column of pixels in which the tasks are drawn
            let column = draw_tasks(column_tasks, renderer_data);

            //then draw that column into  thebuffer
            for y in 0..column.len() {
                // read columns in reverse vertical order; that way other functions can pretend y=0 is botto of screen
                buffer[(SCREEN_HEIGHT - (y + 1)) * SCREEN_WIDTH + x] = column[y];
            }
        }
    }
}

// draws a renderer task to actual pixels
fn draw_tasks(
    column_tasks: &mut ColumnTasks,
    renderer_data: &RendererData,
) -> [u32; SCREEN_HEIGHT] {
    let mut screen_column: [u32; SCREEN_HEIGHT] = [renderer_data.background_color; SCREEN_HEIGHT]; // initialize with default value

    while let Some(task_ord) = column_tasks.tasks.pop() {
        let task = task_ord.task;
        let (onscreen_bottom, onscreen_top) = (
            task.onscreen_bottom
                .clamp(0, renderer_data.screen_height_as_isize),
            task.onscreen_top
                .clamp(0, renderer_data.screen_height_as_isize),
        );
        // try to render a texture, if the task has one
        if let Some(texture_column) = task.texture_column {
            for screen_y in onscreen_bottom..onscreen_top {
                let column_v = screen_y - onscreen_bottom;
                if let Some(pixel_color) = texture_column.get(column_v as usize) {
                    // dont draw outside of screen bounds
                    if screen_y < 0 || screen_y >= renderer_data.screen_height_as_isize {
                        continue;
                    }

                    // 2. Extract channels
                    let a = (pixel_color >> 24) & 0xFF;
                    let r = (pixel_color >> 16) & 0xFF;
                    let g = (pixel_color >> 8) & 0xFF;
                    let b = pixel_color & 0xFF;

                    // if we have transparency at this pixel, we dont draw it; partial transparency not supported
                    if a != 255 {
                        continue;
                    }

                    // 3. Scale each channel with brightness
                    let r = (r as f64 * task.brightness) as u32;
                    let g = (g as f64 * task.brightness) as u32;
                    let b = (b as f64 * task.brightness) as u32;

                    // 4. Repack
                    screen_column[screen_y as usize] = (a << 24) | (r << 16) | (g << 8) | b;
                }
            }
            continue; // go back to beginning of loop, otherwise will get overwritten by color drawing code below
        }

        // render the unicolor instead if the task has no texture
        for onscreen_y in task
            .onscreen_bottom
            .clamp(0, renderer_data.screen_height_as_isize)
            ..task
                .onscreen_top
                .clamp(0, renderer_data.screen_height_as_isize)
        {
            // 2. Extract channels
            let a = (task.color >> 24) & 0xFF;
            let r = (task.color >> 16) & 0xFF;
            let g = (task.color >> 8) & 0xFF;
            let b = task.color & 0xFF;

            // 3. Scale each channel
            let r = (r as f64 * task.brightness) as u32;
            let g = (g as f64 * task.brightness) as u32;
            let b = (b as f64 * task.brightness) as u32;

            // 4. Repack
            screen_column[onscreen_y as usize] = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }

    screen_column
}

///draw refernce points spaced 50 pixels apart for debugging
fn draw_reference_points(buffer: &mut [u32]) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            if x % 50 == 0 && y % 50 == 0 {
                buffer[y * SCREEN_WIDTH + x] = 0xffffff;
            }
        }
    }
}

#[cfg(test)]
mod test {

    // use super::*;
    // #[test]
    // fn test_intersect() {
    //     let ray_origin1 = Point { x: 50.0, y: 200.0 };
    //     let ray_origin2 = Point { x: 50.0, y: 400.0 };
    //     let ray_origin3 = Point { x: 150.0, y: 200.0 };
    //     let ray_origin4 = Point { x: 150.0, y: 400.0 };
    //     let side_point1 = Point { x: 100.0, y: 300.0 };
    //     let side_point2 = Point { x: 100.0, y: 100.0 };
    //     let side = Side {
    //         point1: side_point1,
    //         point2: side_point2,
    //         side_type: ShapeType::Wall,
    //         angle_in_world: 0.0, // isnt used in intersect() anyways
    //         height: LEVEL_HEIGHT,
    //     };
    //     let intersects1 = intersect(ray_origin1, PI, side.clone());
    //     let intersects2 = intersect(ray_origin2, PI, side.clone());
    //     let intersects3 = intersect(ray_origin3, PI, side.clone());
    //     let intersects4 = intersect(ray_origin4, PI, side.clone());
    //     assert!(intersects1.is_none());
    //     assert!(intersects2.is_none());
    //     assert!(intersects3.is_some());
    //     assert!(intersects4.is_none());
    // }

    // #[test]
    // fn test_point_in_polygon () {
    //     let shape = Shape::from_points(
    //         vec![
    //             Point {x: 100.0, y: 100.0},
    //             Point {x: 300.0, y: 100.0},
    //             Point {x: 300.0, y: 300.0},
    //              Point {x: 100.0, y: 300.0},
    //         ],
    //         SideType:: Wall
    //     );

    //     let point_inside1 = Point { x: 200.0, y: 200.0 };
    //     let point_outside1 = Point { x: 10.0, y: 200.0 };
    //     let point_outside2 = Point{x:10.0, y: 10.0};
    //     let inside1 = point_in_polygon(&shape, point_inside1);
    //     let outside1 = point_in_polygon(&shape, point_outside1);
    //     let outside2 = point_in_polygon(&shape, point_outside2);

    //     assert!(inside1);
    //     assert!(!outside1);
    //     assert!(!outside2);

    // }
}
