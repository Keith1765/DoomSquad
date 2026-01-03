use core::f64;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::game::Game;
use crate::game::map::{LEVEL_HEIGHT, Orientation, Point, ShapeType, Side}; // TODO LEVEL_HEIGHT and othe rmap data into sth similar to renderer_data
use crate::render::raycast::{
    self, BlockSlice, MapSlice, RayHit, RayHitOrderer, intersect, raycast,
};
use crate::render::renderer_init::RendererData;
use crate::{BACKGROUND_COLOR, SCREEN_HEIGHT, SCREEN_WIDTH}; // TODO fully move this into renderer_data (currently problem because arraysize wants constant, typing)

type VerticalDisctance = f64;

#[derive(Clone, Copy, PartialEq)]
enum RenderTaskType {
    Side,
    Floor(VerticalDisctance),
    Ceiling(VerticalDisctance),
}
struct RenderTask {
    color: u32, // TODO replace with texture
    brightness: f64,
    onscreen_bottom: isize,
    onscreen_top: isize,
}

struct RenderTaskOrderer {
    pub task: RenderTask,
    task_type: RenderTaskType,
    distance: f64,
}

impl PartialEq for RenderTaskOrderer {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for RenderTaskOrderer {} // PartialEQ already handles functionality, but must be written out; do not remove

impl PartialOrd for RenderTaskOrderer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.distance > other.distance {
            Some(Ordering::Greater)
        } else if self.distance < other.distance {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for RenderTaskOrderer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.task_type, other.task_type) {
            // if both are floors/ceilings, we order by vertical distance
            (RenderTaskType::Ceiling(self_vert_dist), RenderTaskType::Ceiling(other_vert_dist))
            | (RenderTaskType::Floor(self_vert_dist), RenderTaskType::Floor(other_vert_dist)) => {
                if self_vert_dist > other_vert_dist {
                    Ordering::Greater
                } else if self_vert_dist < other_vert_dist {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
            // otherwise, we order by horizontal distance (further back gets drawn first)
            (_, _) => {
                if self.distance > other.distance {
                    Ordering::Greater
                } else if self.distance < other.distance {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
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

pub fn draw(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    //write grey plane as background to overwrite past frames
    for px in buffer.iter_mut() {
        *px = renderer_data.background_color;
    }
    //draw the top down map
    // draw_map(buffer, game).unwrap();
    //go through FOV in small steps, for each draw ray in top down view and corresponding line based on distance in 2.5 view
    draw_camera_view(buffer, &renderer_data, game);
    //draw player with his looking angle
    // draw_player(buffer, game);
    //draw grid of reference points spaced each 50 pixels for debugging
    draw_reference_points(buffer);
}

fn draw_camera_view(buffer: &mut [u32], renderer_data: &RendererData, game: &Game) {
    for x in 0..SCREEN_WIDTH {
        let pixel_distance_from_screen_middle: f64 = x as f64 - SCREEN_WIDTH as f64 / 2.0;
        let angle_relative_to_player: f64 = (pixel_distance_from_screen_middle
            / renderer_data.projection_plane_distance as f64)
            .atan();

        let mut column_tasks: BinaryHeap<RenderTaskOrderer> = task_column(
            game,
            renderer_data,
            angle_relative_to_player,
            game.player.view_angle,
        );

        let column = draw_render_tasks(&mut column_tasks, renderer_data);

        //draw column into buffer
        for y in 0..column.len() {
            // read columns in reverse vertical order; that way other functions can pretend y=0 is botto of screen
            buffer[(SCREEN_HEIGHT - (y + 1)) * SCREEN_WIDTH + x] = column[y];
        }
    }
}

fn draw_render_tasks(
    tasks: &mut BinaryHeap<RenderTaskOrderer>,
    renderer_data: &RendererData,
) -> [u32; SCREEN_HEIGHT] {
    let mut column: [u32; SCREEN_HEIGHT] = [renderer_data.background_color; SCREEN_HEIGHT]; // initialize with default value

    while let Some(task_ord) = tasks.pop() {
        let task = task_ord.task;

        for onscreen_y_isize in task.onscreen_bottom..task.onscreen_top {
            let onscreen_y = onscreen_y_isize as usize; // TODO remove need for this type conversion?

            if onscreen_y >= SCREEN_HEIGHT {
                continue;
            }
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
            column[onscreen_y] = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }

    return column;
}

fn task_column(
    game: &Game,
    renderer_data: &RendererData,
    angle_relative_to_player: f64,
    player_angle: f64,
) -> BinaryHeap<RenderTaskOrderer> {
    let mut tasks: BinaryHeap<RenderTaskOrderer> = BinaryHeap::new();

    let map_slice: MapSlice = raycast(game, angle_relative_to_player, player_angle);

    if let Some(wall_hit) = map_slice.wall_hit {
        tasks.push(task_side(
            &wall_hit,
            angle_relative_to_player,
            renderer_data,
            game,
        )); // default return value: empty column
    }

    for slice in map_slice.bottom_block_slices {
        tasks.append(&mut task_block_slice(
            slice,
            angle_relative_to_player,
            renderer_data,
            game,
        ));
    }

    for slice in map_slice.top_block_slices {
        tasks.append(&mut task_block_slice(
            slice,
            angle_relative_to_player,
            renderer_data,
            game,
        ));
    }

    return tasks;
}

fn task_side(
    side_hit: &RayHit,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> RenderTaskOrderer {
    let (side_bottom_onscreen, side_top_onscreen) =
        calculate_side_bottom_top(&side_hit, angle_relative_to_player, renderer_data, game);

    let color = match &side_hit.side.shape.shape_type {
        ShapeType::Wall => renderer_data.wall_default_color,
        ShapeType::Block(Orientation::Bottom) => renderer_data.bottom_block_default_color,
        ShapeType::Block(Orientation::Top) => renderer_data.top_block_default_color,
    };

    let brightness = (side_hit.side.angle_in_world.cos() * 0.5
        / (side_hit.distance as f64 * renderer_data.distance_darkness_coefficient)
        + 0.5)
        .clamp(0.2, 1.0);

    let task = RenderTask {
        color,
        brightness,
        onscreen_bottom: side_bottom_onscreen,
        onscreen_top: side_top_onscreen,
    };

    return RenderTaskOrderer::new(task, side_hit.distance, RenderTaskType::Side);
}

fn task_surface(
    slice: BlockSlice,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> RenderTaskOrderer {
    let (exit_bottom_onscreen, exit_top_onscreen) = calculate_side_bottom_top(
        &slice.exit_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    );
    let (entry_bottom_onscreen, entry_top_onscreen) = calculate_side_bottom_top(
        &slice.entry_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    );

    let (surface_onscreen_bottom, surface_onscreen_top): (isize, isize) =
        match &slice.entry_hit.side.shape.shape_type {
            ShapeType::Block(Orientation::Bottom) => (entry_top_onscreen, exit_top_onscreen),
            ShapeType::Block(Orientation::Top) => (exit_bottom_onscreen, entry_bottom_onscreen),
            ShapeType::Wall => (0, 0), // null value, shoud never happen
        };

    let vertical_distance: VerticalDisctance = match &slice.entry_hit.side.shape.shape_type {
        ShapeType::Block(Orientation::Bottom) => {
            game.player.view_height - &slice.entry_hit.side.shape.height
        }
        ShapeType::Block(Orientation::Top) => {
            (LEVEL_HEIGHT - &slice.entry_hit.side.shape.height) - game.player.view_height
        }
        ShapeType::Wall => 0.0, // null value, shoud never happen
    };

    // varies between 0.5 and 1.0 depending on height in level; temporary
    let brightness = 0.5 + (&slice.entry_hit.side.shape.height / LEVEL_HEIGHT) * 0.5;

    let task = RenderTask {
        color: renderer_data.surface_default_color,
        brightness,
        onscreen_bottom: surface_onscreen_bottom,
        onscreen_top: surface_onscreen_top,
    };

    let task_type = match &slice.entry_hit.side.shape.shape_type {
        ShapeType::Block(Orientation::Bottom) => RenderTaskType::Floor(vertical_distance),
        ShapeType::Block(Orientation::Top) => RenderTaskType::Ceiling(vertical_distance),
        ShapeType::Wall => RenderTaskType::Floor(0.0), // null value, shoud never happen
    };

    return RenderTaskOrderer::new(task, slice.exit_hit.distance, task_type);
}

fn task_block_slice(
    slice: BlockSlice,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> BinaryHeap<RenderTaskOrderer> {
    let mut tasks = BinaryHeap::new();

    tasks.push(task_side(
        &slice.entry_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    ));

    tasks.push(task_surface(
        slice,
        angle_relative_to_player,
        renderer_data,
        game,
    ));

    tasks
}

fn calculate_side_bottom_top(
    rh: &RayHit,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> (isize, isize) {
    let normalized_distance_to_side = rh.distance * angle_relative_to_player.cos(); // cos for anti-fisheye effect

    let side_height_onscreen = ((rh.side.shape.height / normalized_distance_to_side)
        * renderer_data.vertical_scale_coefficient) as isize; // must be addable to bottom_onscreen

    let mut side_bottom_onscreen: isize = match rh.side.shape.shape_type {
        ShapeType::Wall | ShapeType::Block(Orientation::Bottom) => {
            ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
                - (game.player.view_height / normalized_distance_to_side) // adjust for view hieght
                    * renderer_data.vertical_scale_coefficient) as isize // scale correctly
        } // must be able to be negative
        ShapeType::Block(Orientation::Top) => {
            ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
                + ((LEVEL_HEIGHT - game.player.view_height) // adjust for view height (from top this time)
                    / normalized_distance_to_side)
                    * renderer_data.vertical_scale_coefficient) as isize // scale
                    - side_height_onscreen // move so top flush with level top
        } // must be able to be negative
    };

    let side_top_onscreen =
        (side_bottom_onscreen + side_height_onscreen).min(SCREEN_HEIGHT as isize);
    side_bottom_onscreen = side_bottom_onscreen.max(0);

    (side_bottom_onscreen, side_top_onscreen)
}

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
    use std::vec;

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
