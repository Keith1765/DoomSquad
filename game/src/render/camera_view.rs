use core::f64;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::game::Game;
use crate::render::blocks_walls::task_column;
// TODO LEVEL_HEIGHT and other map data into sth similar to renderer_data?
use crate::render::raycast::{MapSlice, raycast};
use crate::render::renderer_init::RendererData;
use crate::render::sprites::task_sprite;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH}; // TODO fully move this into renderer_data (currently problem because arraysize wants constant, typing)

pub type VerticalDisctance = f64;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderTaskType {
    Floor(VerticalDisctance), // vert dist is needed for sorting between surface tasks
    Ceiling(VerticalDisctance),
    SpriteUnicolor,
    SideTexture,
}

#[derive(Clone)]
pub struct RenderTask {
    pub texture_column: Option<Vec<u32>>, // texture and color will never both be used
    pub color: u32,
    pub brightness: f64,
    pub onscreen_bottom: isize,
    pub onscreen_top: isize,
}

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

impl PartialOrd for RenderTaskOrderer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // surfaces should be rendered above sides when they are of equal (within floating point error) distance, to prevent flickering between the two
        if (self.distance - other.distance).abs() < 0.01 {
            match (self.task_type, other.task_type) {
                // if both are ciling or both are floor, render the one that is vertically closer above further one (prevents flickering between surface of equal distance)
                (RenderTaskType::Ceiling(s_vert_dist), RenderTaskType::Ceiling(o_vert_dist))
                | (RenderTaskType::Floor(s_vert_dist), RenderTaskType::Floor(o_vert_dist)) => {
                    return s_vert_dist.partial_cmp(&o_vert_dist);
                }
                // if both are surfaces (ceilings or floors), but not of same type, order as normal
                (RenderTaskType::Ceiling(_), RenderTaskType::Floor(_))
                | (RenderTaskType::Floor(_), RenderTaskType::Ceiling(_)) => {
                    return self.distance.partial_cmp(&other.distance);
                }
                // if self is a surface but not other, render self above other
                (RenderTaskType::Floor(_), _) | (RenderTaskType::Ceiling(_), _) => {
                    return Some(Ordering::Less);
                }
                // if other is a surface but not self, render other above self
                (_, RenderTaskType::Floor(_)) | (_, RenderTaskType::Ceiling(_)) => {
                    return Some(Ordering::Greater);
                }
                // in all other cases (if neither are surfaces), render in normal ordering
                (_, _) => return self.distance.partial_cmp(&other.distance),
            }
        } else {
            // if not very close, order as one would expect, accorfding to distance
            return self.distance.partial_cmp(&other.distance);
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

pub struct ColumnTasks {
    pub tasks: BinaryHeap<RenderTaskOrderer>,
    pub wall_distance: f64,
}

pub fn draw_screen(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    //write grey plane as background to overwrite past frames
    for px in buffer.iter_mut() {
        *px = renderer_data.background_color;
    }
    draw_camera_view(buffer, &renderer_data, game);
    //draw grid of reference points spaced each 50 pixels for debugging
    if game.player.godmode {
        draw_reference_points(buffer);
    }
    //draw_texture_bottom_left(buffer, renderer_data.textures.get(&0).unwrap());
}

fn draw_camera_view(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    // for every column of the screen, create a slice of the map
    let mut map_slices_and_angles: [Option<(MapSlice, f64)>; SCREEN_WIDTH] =
        [const { None }; SCREEN_WIDTH];
    for x in 0..SCREEN_WIDTH {
        let pixel_distance_from_screen_middle: f64 = x as f64 - SCREEN_WIDTH as f64 / 2.0;
        let angle_relative_to_player: f64 = (pixel_distance_from_screen_middle
            / renderer_data.render_scale_coefficient as f64)
            .atan();

        map_slices_and_angles[x] = Some((
            raycast(
                game,
                angle_relative_to_player,
                game.player.mover.facing_direction,
            ),
            angle_relative_to_player,
        ));
    }

    // convert every mapslice into taskings
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
        if let Some(mut instruction) = task_sprite(game, &entity.sprite, &entity.mover, renderer_data) {
            for x in instruction.sprite_left_screen_x..instruction.sprite_right_screen_x {
                if x < 0 || x > SCREEN_WIDTH - 1 {
                    continue;
                }

                if let Some(cts) = &mut columns_tasked[x]
                    && let Some(sprite_task) =
                        instruction.tasks.pop()
                    && sprite_task.distance <= cts.wall_distance
                {
                    cts.tasks.push(sprite_task);
                }
            }
        }
    }

    // create entity (sprite) tasks, put them into the taskings
    // practically identical to above
    for interactable in &game.interactables {
        if let Some(mut instruction) = task_sprite(game, &interactable.sprite, &interactable.mover, renderer_data) {
            for x in instruction.sprite_left_screen_x..instruction.sprite_right_screen_x {
                if x < 0 || x > SCREEN_WIDTH - 1 {
                    continue;
                }

                if let Some(cts) = &mut columns_tasked[x]
                    && let Some(sprite_task) =
                        instruction.tasks.pop()
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
            let column = draw_tasks(column_tasks, renderer_data);

            //draw column into buffer
            for y in 0..column.len() {
                // read columns in reverse vertical order; that way other functions can pretend y=0 is botto of screen
                buffer[(SCREEN_HEIGHT - (y + 1)) * SCREEN_WIDTH + x] = column[y];
            }
        }
    }
}

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
            //println!("{}",texture_column.len());
            for screen_y in onscreen_bottom..onscreen_top {
                if onscreen_top > renderer_data.screen_height_as_isize {
                    println!("hi");
                }
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

        // render the color instead if the task has no texture
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

// // TODO add positioning to make actualyl useful
// // TODO this whole thing is temporary mostly
// fn draw_texture_bottom_left(buffer: &mut [u32], texture: &Texture) {
//     for x in 0..texture.width {
//         let column = texture.get_column(x).unwrap(); // ! TODO get rid of unwrap
//         for y in 0..column.len() - 1 {
//             buffer[(y * SCREEN_WIDTH) + x] = *column.get(y).unwrap(); // ! TODO get rid of unwrap
//         }
//     }
// }

//draw refernce points spaced 50 pixels apart for debugging
fn draw_reference_points(buffer: &mut [u32]) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            if x % 50 == 0 && y % 50 == 0 {
                buffer[y * SCREEN_WIDTH + x] = 0xffffff;
            }
        }
    }
}

////! this func is a random chatgbt function, rewrite if we want to use it in the final code
fn draw_line(buffer: &mut [u32], x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    // Convert to signed for math (avoids underflow)
    let mut x0 = x0 as isize;
    let mut y0 = y0 as isize;
    let x1 = x1 as isize;
    let y1 = y1 as isize;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Only draw inside the screen
        if x0 >= 0 && x0 < SCREEN_WIDTH as isize && y0 >= 0 && y0 < SCREEN_HEIGHT as isize {
            buffer[y0 as usize * SCREEN_WIDTH + x0 as usize] = color;
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// fn point_in_polygon (shape: &Option<Shape>, point: Point) -> Option<bool> {
//     // let mut side1 : Side;
//     let shape_content: Shape = (*shape).clone()?; // TODO remove necessity for clone() maybe?
//     let mut intersects = false;
//     for side in shape_content.sides {
//         if intersect(point, 0.0, side).is_some() {
//             intersects=!intersects;
//         }
//     }
//     Some(intersects)
// }

// fn draw_dimensional_cast(
//     buffer: &mut [u32],
//     distance_to_wall: f64,
//     ray_angle_relative_to_player_angle: f64,
//     angle_of_wall: f64,
// ) {
//     let normalized_distance_to_wall =
//         (distance_to_wall * ray_angle_relative_to_player_angle.cos()) / WALLSCALING; // cos for anti-fisheye effect

//     let wall_heigth = (HEIGHT as f64 / normalized_distance_to_wall).clamp(0.0, HEIGHT as f64);
//     //find out what ray we are currently casting to know where on the x axis to draw the line in the 2.5 view
//     let center_x = WIDTH as f64 * 0.5;
//     let proj_dist = center_x / (FOV * 0.5).tan();
//     let x = (center_x + ray_angle_relative_to_player_angle.tan() * proj_dist) as usize;

//     let draw_srting_point = (HEIGHT as f64 - wall_heigth) / 2.0;

//     //draw the vertical line; shading based on angle of the side
//     for y in
//         draw_srting_point as usize..(draw_srting_point + wall_heigth).min(HEIGHT as f64) as usize
//     {
//         let brightness = (angle_of_wall.cos() * 0.5 + 0.5).clamp(0.2, 1.0);
//         let color = 0x00ff00;
//         // 2. Extract channels
//         let a = (color >> 24) & 0xFF;
//         let r = (color >> 16) & 0xFF;
//         let g = (color >> 8) & 0xFF;
//         let b = color & 0xFF;

//         // 3. Scale each channel
//         let r = (r as f64 * brightness) as u32;
//         let g = (g as f64 * brightness) as u32;
//         let b = (b as f64 * brightness) as u32;

//         // 4. Repack

//         buffer[y * WIDTH + x] = (a << 24) | (r << 16) | (g << 8) | b;
//     }
// }

//save all points from the screen that are in the polygon of the map boarder and note that map is loaded now
////! right now load map is working not as intended in the game, because right now it loads the init of map, so right now it just means that we init the map, later however it will indicate what map was loaded into the map boarder
// fn load_map (game: & mut Game) -> Result<(),Box<dyn  std::error::Error>>{
//     for x in 0..WIDTH {
//         for y in 0..HEIGHT{
//             if point_in_polygon(&game.map.walls, Point { x: x as f64, y: y as f64 }){
//                 game.map.points_in_border.push(Point{x: x as f64, y: y as f64});
//             }
//         }
//     }
//     game.map.loaded_map=1;
//     Ok(())
// }

//draw the top down view of the map init
// fn draw_map (buffer: &mut [u32], game: &Game) -> Result<(),Box<dyn  std::error::Error>>{
//     for points in game.map.points_in_border.clone() {
//             if point_in_polygon(&game.map.walls, Point { x: points.x as f64, y: points.y as f64 }){
//                 buffer[points.y as usize*WIDTH+points.x as usize] = 0x00ff00;
//         }
//         //draw object
//         if point_in_polygon(&game.map.blocks.get(0).unwrap(), Point { x: points.x as f64, y: points.y as f64 }){
//                 buffer[points.y as usize*WIDTH+points.x as usize] = 0x0000ff;
//         }
//     }
//     Ok(())
// }

// fn draw_player (buffer: &mut [u32], game: &Game) {
//     //make player thicccker but have to check for out of bounds
//     let x = game.player.position_x as isize;
//     let y = game.player.position_y as isize;

//     for dx in -1..=1 {
//         for dy in -1..=1 {
//             let px = x + dx;
//             let py = y + dy;

//             if px >= 0 && px < WIDTH as isize && py >= 0 && py < HEIGHT as isize {
//                 let ux = px as usize;
//                 let uy = py as usize;
//                 let index = uy * WIDTH + ux;
//                 buffer[index] = 0xff0000;
//             }
//         }
//     }
//     //draw direction of player looking as a small line
//     let x1f = game.player.position_x+game.player.velocity_x*5.0;
//     let y1f = game.player.position_y+game.player.velocity_y*5.0;

//     let x1 = x1f.clamp(0.0, (WIDTH - 1) as f64) as usize;
//     let y1 = y1f.clamp(0.0, (HEIGHT - 1) as f64) as usize;
//     let x0 = game.player.position_x.clamp(0.0, (WIDTH-1) as f64) as usize;
//     let y0 = game.player.position_y.clamp(0.0, (HEIGHT-1) as f64) as usize;

//     draw_line(buffer, x0, y0, x1, y1, 0x00ffff);
// }

//

//draw the vertical line for the ray that renders the the 2.5 view

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
