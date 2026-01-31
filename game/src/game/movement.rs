use std::rc::Rc;

use crate::game::{
    Game,
    map::{Map, Point, Side},
};

#[derive(Clone)]
pub struct Mover {
    pub position: Point,
    pub floor_level: f64,
    pub foot_level: f64,
    pub view_level: f64,
    pub height: f64,
    pub facing_direction: f64,
}

impl Mover {
    pub fn step(&mut self, step_size: f64, relative_direction: f64, map: &Map, godmode: bool) {
        let absolute_direction = self.facing_direction + relative_direction;

        // if we walk into any wall and arent in godmode, we dont move
        for wall in &map.wall_sides {
            if let Some(_) = step_intersect(
                self.position,
                absolute_direction,
                Rc::clone(wall),
                step_size,
            ) && !godmode
            {
                return;
            }
        }

        let step_x = absolute_direction.cos() * step_size;
        let step_y = absolute_direction.sin() * step_size;
        self.position = self.position
            + Point {
                x: step_x,
                y: step_y,
            }
    }
}

pub struct StepRayHit {
    pub position: Point,
    pub distance: f64, // TODO remove, not needed ?
    pub side: Rc<Side>,
}

//checks wether a ray intersect the line between two given points and is closer than given distance
//mostly a copy of intersect() in raycast.rs
pub fn step_intersect(
    ray_origin: Point,
    ray_angle: f64,
    side: Rc<Side>,
    max_distance: f64,
) -> Option<StepRayHit> {
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
    let distance: f64 = (side_point2_transformed.x - side_point1_transformed.x) * proportion
        + side_point1_transformed.x; // distance between player and intersect

    if distance < 0.0 || distance > max_distance {
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
    Some(StepRayHit {
        position,
        distance,
        side: Rc::clone(&side),
    })
}

fn rotate_point_around_origin(point: Point, angle: f64) -> Point {
    let sin_of_angle = angle.sin();
    let cos_of_angle = angle.cos();

    let transformed_x = point.x * cos_of_angle - point.y * sin_of_angle;
    let transformed_y = point.x * sin_of_angle + point.y * cos_of_angle;

    Point {
        x: transformed_x,
        y: transformed_y,
    }
}
