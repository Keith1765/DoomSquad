use core::f64;
use std::f64::consts::PI;

use crate::game::map::{Point, Shape};
use crate::{HEIGHT, WIDTH};
use crate::game::{Game};
const FOV : f64 = PI/2.09;
const RENDERSCREENPOINT: Point = Point{x:400.0, y:0.0};
const WALLSCALING : f64 = 23.0;

////!unsafe just for testing, later remove unwrap
pub fn draw(buffer: &mut [u32], game: & mut Game) {
    //write grey plane as background to overwrite past player position
    for px in buffer.iter_mut() {
        *px = 0x222222;
    }
    //load map if not loaded, this safes all points in vec all other points are not used anymore for rendering
    if game.map.loaded_map==0 {
        load_map(game).unwrap();
    }
    //draw the top down map
    draw_map(buffer, game).unwrap();
    //go through FOW in small steps, for each draw ray in top down view and corresponding line based on distance in 2.5 view
    let mut angle = -FOV/2.0;
    let step = 0.0005;
    while angle < (FOV/2.0){
        draw_raycast(buffer, game, angle);
        angle += step;
    }
    //draw player with his looking angle
    draw_player(buffer, game);
    //draw grid of reference points spaced each 50 pixels for debugging
    draw_reference_points (buffer).unwrap();
}

//save all points from the screen that are in the polygon of the map boarder and note that map is loaded now
////! right now load map is working not as intended in the game, because right now it loads the init of map, so right now it just means that we init the map, later however it will indicate what map was loaded into the map boarder
fn load_map (game: & mut Game) -> Result<(),Box<dyn  std::error::Error>>{
    for x in 0..WIDTH {
        for y in 0..HEIGHT{
            if point_in_polygon(&game.map.border, Point { x: x as f64, y: y as f64 }){
                game.map.points_in_border.push(Point{x: x as f64, y: y as f64});
            }
        }
    }
    game.map.loaded_map=1;
    Ok(())
}

//draw refernce points spaced 50 pixels apart for debugging
fn draw_reference_points (buffer: &mut [u32]) -> Result<(),Box<dyn  std::error::Error>>{
    for x in 0..WIDTH {
        for y in 0..HEIGHT{
            if x %50 == 0 && y % 50 == 0{
                buffer[y*WIDTH+x] = 0xff0000;
            }
        }
    }
    Ok(())
}

//draw the top down view of the map init
fn draw_map (buffer: &mut [u32], game: &Game) -> Result<(),Box<dyn  std::error::Error>>{
    for points in game.map.points_in_border.clone() {
            if point_in_polygon(&game.map.border, Point { x: points.x as f64, y: points.y as f64 }){
                buffer[points.y as usize*WIDTH+points.x as usize] = 0x00ff00;
        }
        //draw object
        if point_in_polygon(&game.map.objects.get(0).unwrap(), Point { x: points.x as f64, y: points.y as f64 }){
                buffer[points.y as usize*WIDTH+points.x as usize] = 0x0000ff;
        }
    }
    Ok(())
}


fn draw_player (buffer: &mut [u32], game: &Game) {
    //make player thicccker but have to check for out of bounds
    let x = game.player.px as isize;
    let y = game.player.py as isize;

    for dx in -1..=1 {
        for dy in -1..=1 {
            let px = x + dx;
            let py = y + dy;

            if px >= 0 && px < WIDTH as isize && py >= 0 && py < HEIGHT as isize {
                let ux = px as usize;
                let uy = py as usize;
                let index = uy * WIDTH + ux;
                buffer[index] = 0xff0000;
            }
        }
    }
    //draw direction of player looking as a small line
    let x1f = game.player.px+game.player.pdx*5.0;
    let y1f = game.player.py+game.player.pdy*5.0;

    let x1 = x1f.clamp(0.0, (WIDTH - 1) as f64) as usize;
    let y1 = y1f.clamp(0.0, (HEIGHT - 1) as f64) as usize;
    let x0 = game.player.px.clamp(0.0, (WIDTH-1) as f64) as usize;
    let y0 = game.player.py.clamp(0.0, (HEIGHT-1) as f64) as usize;

    draw_line(buffer, x0, y0, x1, y1, 0x00ffff);
}

fn draw_raycast (buffer: &mut [u32], game: & Game, ray_angle_relative_to_player_angle: f64) {
    
    let mut point1 : Point;
    let mut point2: Point  = *game.map.border.points.last().unwrap();
    let mut intersected_sides: Vec<(f64,f64)> = Vec::new();
    let ray_angle= game.player.pa+ray_angle_relative_to_player_angle;
    //collect intersects with map boarder edges
    for i in 0..game.map.border.points.len(){
        point1 = point2;
        point2 = *game.map.border.points.get(i).unwrap();
        let intersect_value = intersect(Point { x: game.player.px, y: game.player.py }, ray_angle, point1, point2);
        if intersect_value.0{
            intersected_sides.push((intersect_value.1,intersect_value.2));
        }
    }

    //collect intersects with object (only one)
    for i in 0..game.map.objects.get(0).unwrap().points.len(){
        point1 = point2;
        point2 = *game.map.objects.get(0).unwrap().points.get(i).unwrap();
        let intersect_value = intersect(Point { x: game.player.px, y: game.player.py }, ray_angle, point1, point2);
        if intersect_value.0{
            intersected_sides.push((intersect_value.1,intersect_value.2));
        }
    }

    //find shortest intersect and save it
    let mut distance_to_wall: f64=4000.0;
    let mut angle_of_wall: f64 = 0.0;
    
    let mut y:(f64,f64);
    if let Some(t) = intersected_sides.first() {
        y = *t;
    }
    for i in 0..intersected_sides.len(){
        y=*intersected_sides.get(i).unwrap();
        if distance_to_wall > y.0 {
            distance_to_wall= y.0;
            angle_of_wall = y.1;
        }

    }

    //find the point the ray intersects the wall
    let ray_dx = ray_angle.cos();
    let ray_dy =ray_angle.sin();

    let wall_point = Point{x:game.player.px+ray_dx*distance_to_wall ,y:game.player.py + ray_dy*distance_to_wall};
    //draw the line of this ray up to its intersect
    draw_line(buffer, game.player.px as usize, game.player.py as usize, wall_point.x as usize, wall_point.y as usize, 0xff0000);
    //draw the line for the ray in the 2.5 view
    draw_dimensional_cast(buffer, game, distance_to_wall,ray_angle_relative_to_player_angle, angle_of_wall);
}

fn draw_dimensional_cast (buffer: &mut [u32], game: & Game, distance_to_wall: f64, ray_angle_relative_to_player_angle: f64, angle_of_wall: f64){
    
    let normalized_distance_to_wall = (distance_to_wall * ray_angle_relative_to_player_angle.cos())/WALLSCALING;
    
    let wall_heigth = (RENDERSCREENPOINT.x as f64 / normalized_distance_to_wall ).clamp(1.0, RENDERSCREENPOINT.x);
    //find out what ray we are currently casting to know where on the x axis to draw the line in the 2.5 view
    let draw_percent_based_on_angle = (FOV/2.0+ray_angle_relative_to_player_angle)/FOV.clamp(0.0, 1.0);

    let draw_srting_point = (RENDERSCREENPOINT.x - wall_heigth)/2.0;

    let x = (RENDERSCREENPOINT.x + (RENDERSCREENPOINT.x * draw_percent_based_on_angle)) as usize;

    //shading based on angle of the side
    for y in draw_srting_point as usize..(draw_srting_point+wall_heigth).min(400.0) as usize{
        let brightness = (angle_of_wall.cos() * 0.5 + 0.5).clamp(0.2, 1.0);
        let color = 0x00ff00;
            // 2. Extract channels
        let a = (color >> 24) & 0xFF;
        let r = (color >> 16) & 0xFF;
        let g = (color >> 8)  & 0xFF;
        let b =  color        & 0xFF;

        // 3. Scale each channel
        let r = (r as f64 * brightness) as u32;
        let g = (g as f64 * brightness) as u32;
        let b = (b as f64 * brightness) as u32;

        // 4. Repack
        
        buffer[y*WIDTH+x] = (a << 24) | (r << 16) | (g << 8) | b;

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

//checks wether a ray intersect the line between two given points
fn intersect(ray_origin_point: Point, ray_angle: f64, side_point1: Point, side_point2: Point) -> (bool, f64, f64){
    

    //makes ray_point origin (=(0|0))
    let side_point1_relative = side_point1-ray_origin_point;
    let side_point2_relative = side_point2-ray_origin_point;

    let sin_of_angle = (-ray_angle).sin();
    let cos_of_angle = (-ray_angle).cos();
    //rotates points so that the ray angle is 0
    let  side_point1_transformed = rotate_point_around_origin(side_point1_relative, sin_of_angle, cos_of_angle);
    let  side_point2_transformed = rotate_point_around_origin(side_point2_relative, sin_of_angle, cos_of_angle);

    // if side_point1_transformed.y < side_point2_transformed.y {
    //     return false;
    // }

    //lemme cook Jakob
    // if side_point1_transformed.y < side_point2_transformed.y {
    //     std::mem::swap(& mut side_point1_transformed, & mut side_point2_transformed);
    // }

    

    if (side_point1_transformed.y > 0.0) == (side_point2_transformed.y > 0.0) {
        return (false,0.0,0.0);
    }
    
    let proportion = -side_point1_transformed.y / (side_point2_transformed.y-side_point1_transformed.y);
    let distance_to_intersect = (side_point2_transformed.x-side_point1_transformed.x)*proportion + side_point1_transformed.x;

    if distance_to_intersect < 0.0 {
        return (false,0.0,0.0);
    }

    let angle = (side_point2.y-side_point1.y).atan2(side_point2.x-side_point1.x);
    return (true,distance_to_intersect,angle);
}

fn rotate_point_around_origin (point: Point, sin_of_angle: f64, cos_of_angle: f64) -> Point {
    
    let transformed_x = point.x * cos_of_angle - point.y * sin_of_angle;
    let transformed_y = point.x * sin_of_angle + point.y * cos_of_angle;
    
    return Point { x: transformed_x, y: transformed_y };
}

fn point_in_polygon (shape: &Shape, point: Point) -> bool{
    let mut point1 : Point;
    let mut point2: Point  = *shape.points.last().unwrap();
    let mut intersects = false;
    for i in 0..shape.points.len() {
        point1 = point2;
        point2 = *shape.points.get(i).unwrap();
        if intersect(point, 0.0, point1, point2).0 {intersects=!intersects;}
    }
    intersects
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_intersect () {
        let ray_origin1 = Point{x:50.0, y: 200.0};
        let ray_origin2 = Point{x:50.0, y: 400.0};
        let ray_origin3 = Point{x:150.0, y: 200.0};
        let ray_origin4 = Point{x:150.0, y: 400.0};
        let side_point1 = Point{x:100.0, y: 300.0};
        let side_point2 = Point{x:100.0, y: 100.0};
        let intersects1 = intersect(ray_origin1, 0.0, side_point1, side_point2);
        let intersects2 = intersect(ray_origin2, 0.0, side_point1, side_point2);
        let intersects3 = intersect(ray_origin3, 0.0, side_point1, side_point2);
        let intersects4 = intersect(ray_origin4, 0.0, side_point1, side_point2);
        assert!(intersects1.0);
        assert!(!intersects2.0);
        assert!(!intersects3.0);
        assert!(!intersects4.0);
    }

    #[test]
    fn test_point_in_polygon () {
        let shape = Shape{
                points: vec![
                    Point {x: 100.0, y: 100.0},
                    Point {x: 300.0, y: 100.0},
                    Point {x: 300.0, y: 300.0},
                    Point {x: 100.0, y: 300.0},
                ]
        };

        let point_inside1 = Point { x: 200.0, y: 200.0 };
        let point_outside1 = Point { x: 10.0, y: 200.0 };
        let point_outside2 = Point{x:10.0, y: 10.0};
        let inside1 = point_in_polygon(&shape, point_inside1);
        let outside1 = point_in_polygon(&shape, point_outside1);
        let outside2 = point_in_polygon(&shape, point_outside2);

        assert!(inside1);
        assert!(!outside1);
        assert!(!outside2);


    }

}