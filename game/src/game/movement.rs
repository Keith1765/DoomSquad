use crate::game::map::{Map, Point, Shape, Side};
use crate::game::player::MAX_STEP_UP_HEIGHT;
use core::f64;
use std::{collections::HashSet, rc::Rc};

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
    // TODO test this properly
    // returns whether a step was made or blocked
    pub fn step(
        &mut self,
        step_size: f64,
        relative_direction: f64,
        map: &Map,
        godmode: bool,
    ) -> bool {
        //println!("{}", self.floor_level);

        let absolute_direction = self.facing_direction + relative_direction;

        // if we walk into any wall and arent in godmode, we stop
        for w in &map.wall_sides {
            if let Some(_) =
                step_intersect(self.position, absolute_direction, Rc::clone(w), step_size)
                && !godmode
            // in godmode, we can walk through walls
            {
                return false;
            }
        }

        let step_x = absolute_direction.cos() * step_size;
        let step_y = absolute_direction.sin() * step_size;
        let new_position = self.position
            + Point {
                x: step_x,
                y: step_y,
            };

        // we find all the sides which we would cross in the coming step
        // if we are in godmode, we just go through everything, so we disregard everything
        let blocks_were_stepping_inside = find_blocks_were_currently_in(new_position, map);

        let mut lowest_ceiling_level = f64::MAX;
        let mut height_to_step_to = 0.0;
        for block in &blocks_were_stepping_inside {
            let block_bottom = block.bottom;
            let block_top = block.bottom + block.height;
            let head_level = self.foot_level + self.height;

            // we check if the current side blocks our path completely; if so, we dont make a step at all
            if block_bottom <= head_level // bottom is below our head
                && block_top > self.foot_level + MAX_STEP_UP_HEIGHT
            // cant step up the side
            {
                //println!("blocked completely");
                if !godmode {
                    return false;
                } // if we are in godmode, we dont let ourselves get blocked
                continue;
            }

            if block_bottom < lowest_ceiling_level // lower thn lowest previously found ceiling level
                && block_top > self.foot_level + MAX_STEP_UP_HEIGHT
            // block cant be stepped up onto
            {
                lowest_ceiling_level = block_bottom;
            }

            // checks for steps up ledges
            if block_bottom <= head_level // bottom below our head (not totally out of way)
                && block_top <= self.foot_level + MAX_STEP_UP_HEIGHT // can step up the side
                && block_top > height_to_step_to
            // would be a higher step than any previous steps up; if not, irrelevant, do nothing
            {
                //println!("up {}", block.id);
                height_to_step_to = block_top;
                continue;
            }
        }

        // if our new floor level would result in us bumping our head into the ceiling, we dont make a step
        if height_to_step_to + self.height >= lowest_ceiling_level {
            return false;
        }

        self.position = new_position;
        self.floor_level = height_to_step_to;

        true
    }
}

// finds all block we are currently staning inside of in 2d space
// if we intersect a blocks sides an even number of times, we are outside of it, if odd, we are inside
pub fn find_blocks_were_currently_in(position: Point, map: &Map) -> Vec<Rc<Shape>> {
    let mut blocks_currently_inside: HashSet<Rc<Shape>> = HashSet::new();
    for side in &map.block_sides {
        if step_intersect(position, 0.0, Rc::clone(side), f64::MAX).is_some() {
            // if its already in set, this was an even-numbered inteersect and we want to remove again
            if blocks_currently_inside.contains(&Rc::clone(&side.shape)) {
                blocks_currently_inside.remove(&Rc::clone(&side.shape));
            // if its not in set, we conversely need to add it (back) in
            } else {
                blocks_currently_inside.insert(Rc::clone(&side.shape));
            }
        }
    }
    Vec::from_iter(blocks_currently_inside)
}

pub struct StepRayHit {
    pub position: Point,
    pub distance: f64, // TODO remove, not needed ?
    pub side: Rc<Side>,
}

//checks wether a ray intersect the line between two given points
// adapted from intersect() in raycast.rs
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

// TODO tests fro blocks_were_inside() etc
