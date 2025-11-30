use core::f64;
use std::f64::consts::PI;

use crate::game::Game;
use crate::game::map::{Point, Shape, Side, SideType};
use crate::{HEIGHT, WIDTH};
const FOV: f64 = PI / 2.09;
const WALLSCALING: f64 = 23.0;
const BACKGROUND_COLOR: u32 = 0x222222;

////!unsafe just for testing, later remove unwrap
pub fn draw(buffer: &mut [u32], game: &Game) {
    //write grey plane as background to overwrite past frames
    for px in buffer.iter_mut() {
        *px = BACKGROUND_COLOR;
    }
    //draw the top down map
    // draw_map(buffer, game).unwrap();
    //go through FOW in small steps, for each draw ray in top down view and corresponding line based on distance in 2.5 view
    draw_screen(buffer, game);
    //draw player with his looking angle
    // draw_player(buffer, game);
    //draw grid of reference points spaced each 50 pixels for debugging
    draw_reference_points(buffer).unwrap();
}

fn draw_screen(buffer: &mut [u32], game: &Game) {
    let mut angle_relative_to_player = -FOV / 2.0;
    let step = FOV/WIDTH as f64;
    for x in 0..WIDTH {
        let column: [u32; HEIGHT] = get_drawing_column(
            game,
            angle_relative_to_player,
            game.player.view_angle,
        );
        //draw column into buffer
        for y in 0..column.len() {
            buffer[y*WIDTH+x] = column[y];
        }
        angle_relative_to_player += step;
    }
}

fn get_drawing_column(
    game: &Game,
    angle_relative_to_player: f64,
    player_angle: f64,
) -> [u32; HEIGHT] {
    // // let mut side1 : Side;
    //     let shape_content: Shape = (*shape).clone()?; // TODO remove necessity for clone() maybe?
    //     let mut intersects = false;
    //     for side in shape_content.sides {
    //         if intersect(point, 0.0, side).is_some() {
    //             intersects=!intersects;
    //         }
    //     }
    //     Some(intersects)
    let mut column: [u32; HEIGHT] = [BACKGROUND_COLOR; HEIGHT]; // initialized with default value

    let ray_angle = player_angle + angle_relative_to_player;
    let mut rayhits: Vec<RayHit> = Vec::new();
    //collect intersects with map border edges
    for w in game.map.walls.clone() {
        for s in w.sides {
            let intersection = intersect(
                Point {
                    x: game.player.position_x,
                    y: game.player.position_y,
                },
                ray_angle,
                s,
            );
            if let Some(intersect) = intersection {
                rayhits.push(intersect);
            }
        }
    }
    if rayhits.is_empty() {
        return [BACKGROUND_COLOR; HEIGHT]; // default return value: empty column
    }
    //find closest rayhit and save it
    let mut closest_hit: RayHit;
    if let Some(h) = rayhits.first() {
        closest_hit = h.clone();
        for rh in rayhits {
            if closest_hit.distance > rh.distance {
                closest_hit = rh;
            }
        }

        let normalized_distance_to_wall =
            (closest_hit.distance * angle_relative_to_player.cos()) / WALLSCALING; // cos for anti-fisheye effect
        let wall_heigth = (HEIGHT as f64 / normalized_distance_to_wall).clamp(0.0, HEIGHT as f64);
        //find out what ray we are currently casting to know where on the x axis to draw the line in the 2.5 view
        //let center_x = WIDTH as f64 * 0.5;
        //let proj_dist = center_x / (FOV * 0.5).tan();
        //let x = (center_x + ray_angle_relative_to_player_angle.tan() * proj_dist) as usize;

        // we draw wall bottom to top, with shading
        let side_bottom_screen_y = (HEIGHT as f64 - wall_heigth) / 2.0;
        for y in side_bottom_screen_y as usize
            ..(side_bottom_screen_y + wall_heigth).min(HEIGHT as f64) as usize
        {
            let brightness = (closest_hit.side.angle_in_world.cos() * 0.5 + 0.5).clamp(0.2, 1.0);
            let color = 0x00ff00;
            // 2. Extract channels
            let a = (color >> 24) & 0xFF;
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;

            // 3. Scale each channel
            let r = (r as f64 * brightness) as u32;
            let g = (g as f64 * brightness) as u32;
            let b = (b as f64 * brightness) as u32;

            // 4. Repack
            column[y] = (a << 24) | (r << 16) | (g << 8) | b;
        }
    }

    //find the point the ray intersects the wall
    // let ray_dx = ray_angle.cos();
    // let ray_dy = ray_angle.sin();

    // let wall_point = Point {
    //     x: game.player.position_x + ray_dx * distance_to_wall,
    //     y: game.player.position_y + ray_dy * distance_to_wall,
    // };
    //draw the line of this ray up to its intersect
    //draw_line(buffer, game.player.position_x as usize, game.player.position_y as usize, wall_point.x as usize, wall_point.y as usize, 0xff0000);
    return column;
}

fn draw_dimensional_cast(
    buffer: &mut [u32],
    distance_to_wall: f64,
    ray_angle_relative_to_player_angle: f64,
    angle_of_wall: f64,
) {
    let normalized_distance_to_wall =
        (distance_to_wall * ray_angle_relative_to_player_angle.cos()) / WALLSCALING; // cos for anti-fisheye effect

    let wall_heigth = (HEIGHT as f64 / normalized_distance_to_wall).clamp(0.0, HEIGHT as f64);
    //find out what ray we are currently casting to know where on the x axis to draw the line in the 2.5 view
    let center_x = WIDTH as f64 * 0.5;
    let proj_dist = center_x / (FOV * 0.5).tan();
    let x = (center_x + ray_angle_relative_to_player_angle.tan() * proj_dist) as usize;

    let draw_srting_point = (HEIGHT as f64 - wall_heigth) / 2.0;

    //draw the vertical line; shading based on angle of the side
    for y in
        draw_srting_point as usize..(draw_srting_point + wall_heigth).min(HEIGHT as f64) as usize
    {
        let brightness = (angle_of_wall.cos() * 0.5 + 0.5).clamp(0.2, 1.0);
        let color = 0x00ff00;
        // 2. Extract channels
        let a = (color >> 24) & 0xFF;
        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;

        // 3. Scale each channel
        let r = (r as f64 * brightness) as u32;
        let g = (g as f64 * brightness) as u32;
        let b = (b as f64 * brightness) as u32;

        // 4. Repack

        buffer[y * WIDTH + x] = (a << 24) | (r << 16) | (g << 8) | b;
    }
}

//draw refernce points spaced 50 pixels apart for debugging
fn draw_reference_points(buffer: &mut [u32]) -> Result<(), Box<dyn std::error::Error>> {
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if x % 50 == 0 && y % 50 == 0 {
                buffer[y * WIDTH + x] = 0xff0000;
            }
        }
    }
    Ok(())
}

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
        if x0 >= 0 && x0 < WIDTH as isize && y0 >= 0 && y0 < HEIGHT as isize {
            buffer[y0 as usize * WIDTH + x0 as usize] = color;
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

#[derive(Clone)]
struct RayHit {
    position: Point,
    distance: f64,
    proportion_along_side: f64, // how far of the way from left to right we go along the side
    side: Side,
}

//checks wether a ray intersect the line between two given points
fn intersect(ray_origin: Point, ray_angle: f64, side: Side) -> Option<RayHit> {
    let side_point1 = side.point1; // point is a copy type
    let side_point2 = side.point2;

    // effectively makes ray_point origin (=(0|0))
    let side_point1_relative = side_point1 - ray_origin;
    let side_point2_relative = side_point2 - ray_origin;
    //rotates points so that the ray angle is 0
    let side_point1_transformed = rotate_point_around_origin(side_point1_relative, -ray_angle);
    let side_point2_transformed = rotate_point_around_origin(side_point2_relative, -ray_angle);

    // checks if we are going past the side by checking if x axis intersects between 1.y and 2.y
    if (side_point1_transformed.y > 0.0) == (side_point2_transformed.y > 0.0) {
        return None;
    }

    let proportion =
        -side_point1_transformed.y / (side_point2_transformed.y - side_point1_transformed.y); // gives us how far along the wall we are
    let distance = (side_point2_transformed.x - side_point1_transformed.x) * proportion
        + side_point1_transformed.x; // distance between player and intersect
    if distance < 0.0 {
        // if the side is behind us, no Rayhit
        return None;
    }
    let position_in_trasformed_coords = Point {
        x: distance,
        y: 0.0,
    };
    let position =
        rotate_point_around_origin(position_in_trasformed_coords, ray_angle) + ray_origin;

    // let angle = (side_point2.y-side_point1.y).atan2(side_point2.x-side_point1.x);
    return Some(RayHit {
        position: position,
        distance,
        proportion_along_side: proportion,
        side: side,
    });
}

fn rotate_point_around_origin(point: Point, angle: f64) -> Point {
    let sin_of_angle = angle.sin();
    let cos_of_angle = angle.cos();

    let transformed_x = point.x * cos_of_angle - point.y * sin_of_angle;
    let transformed_y = point.x * sin_of_angle + point.y * cos_of_angle;

    return Point {
        x: transformed_x,
        y: transformed_y,
    };
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

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    #[test]
    fn test_intersect() {
        let ray_origin1 = Point { x: 50.0, y: 200.0 };
        let ray_origin2 = Point { x: 50.0, y: 400.0 };
        let ray_origin3 = Point { x: 150.0, y: 200.0 };
        let ray_origin4 = Point { x: 150.0, y: 400.0 };
        let side_point1 = Point { x: 100.0, y: 300.0 };
        let side_point2 = Point { x: 100.0, y: 100.0 };
        let side = Side {
            point1: side_point1,
            point2: side_point2,
            side_type: SideType::Wall,
            angle_in_world: 0.0, // isnt used in intersect() anyways
        };
        let intersects1 = intersect(ray_origin1, PI, side.clone());
        let intersects2 = intersect(ray_origin2, PI, side.clone());
        let intersects3 = intersect(ray_origin3, PI, side.clone());
        let intersects4 = intersect(ray_origin4, PI, side.clone());
        assert!(intersects1.is_none());
        assert!(intersects2.is_none());
        assert!(intersects3.is_some());
        assert!(intersects4.is_none());
    }

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
