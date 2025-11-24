use std::f64::consts::PI;
use std::f64::EPSILON;

use crate::game::map::{Point, Shape};
use crate::{HEIGHT, WIDTH};
use crate::game::{Game,Map};

pub fn draw(buffer: &mut [u32], game: &Game) {
    //Clear the buffer
    for px in buffer.iter_mut() {
        *px = 0x222222;
    }
    draw_map(buffer, game);
    draw_player(buffer, game);
}


fn draw_map (buffer: &mut [u32], game: &Game) -> Result<(),Box<dyn  std::error::Error>>{
    for x in 0..WIDTH {
        for y in 0..HEIGHT{
            if point_in_polygon(&game.map.border, Point { x: (game.player.px), y: (game.player.py) }){
                buffer[y*WIDTH+x] = 0x00ff00;
            }
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
                buffer[index] = 0xFFFF0000;
            }
        }
    }
    let x1f = game.player.px+game.player.pdx*5.0;
    let y1f = game.player.py+game.player.pdy*5.0;

    let x1 = x1f.clamp(0.0, (WIDTH - 1) as f64) as usize;
    let y1 = y1f.clamp(0.0, (HEIGHT - 1) as f64) as usize;
    let x0 = game.player.px.clamp(0.0, (WIDTH-1) as f64) as usize;
    let y0 = game.player.py.clamp(0.0, (HEIGHT-1) as f64) as usize;

    draw_line(buffer, x0, y0, x1, y1, 0xFFFF0000);
}

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


fn intersect(ray_origin_point: Point, ray_angle: f64, side_point1: Point, side_point2: Point) -> bool{
    

    //makes ray_point origin
    let side_point1_relative = side_point1-ray_origin_point;
    let side_point2_relative = side_point2-ray_origin_point;

    let side_point1_transformed = rotate_point_around_origin(side_point1_relative, -ray_angle);
    let side_point2_transformed = rotate_point_around_origin(side_point2_relative, -ray_angle);

    // if side_point1_transformed.y < side_point2_transformed.y {
    //     return false;
    // }

    

    if side_point1_transformed.y < 0.0 || side_point2_transformed.y > 0.0 {
        return false;
    }
    
    let proportion = side_point2_transformed.y / (side_point2_transformed.y-side_point1_transformed.y);
    let distance_to_intersect = (side_point1_transformed.x-side_point2_transformed.x)*proportion + side_point1_transformed.x;

    if distance_to_intersect < 0.0 {
        return false;
    }

    return true;
}

fn rotate_point_around_origin (point: Point, angle: f64) -> Point {
    let transformed_x = point.x * angle.cos() - point.y * angle.sin();
    let transformed_y = point.x * angle.sin() + point.y * angle.cos();
    
    return Point { x: transformed_x, y: transformed_y };
}

fn point_in_polygon (shape: &Shape, point: Point) -> bool{
    let mut point1 : Point = Point {x: 0.0, y: 0.0};
    let mut point2: Point  = *shape.points.last().unwrap();
    let mut intersects = false;
    for i in 0..shape.points.len() {
        point1 = point2;
        point2 = *shape.points.get(i).unwrap();
        if intersect(point, 0.0, point1, point2) {intersects=!intersects;}
    }
    intersects
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_intersect () {
        let ray_origin = Point{x:10.0, y: 200.0};
        let side_point1 = Point{x:100.0, y: 300.0};
        let side_point2 = Point{x:100.0, y: 100.0};
        let intersects = intersect(ray_origin, 0.0, side_point1, side_point2);
        assert!(intersects);
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

        let point_inside = Point { x: 200.0, y: 200.0 };
        let point_outside = Point { x: 10.0, y: 200.0 };
        let inside = point_in_polygon(&shape, point_inside);
        let outside = point_in_polygon(&shape, point_outside);

        assert!(inside);
        assert!(!outside);

    }

}