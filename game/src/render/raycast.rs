use std::{cmp::Ordering, collections::BinaryHeap};

use crate::game::{Game, map::{Point, Side}};

#[derive(Clone, PartialEq)]
pub struct RayHit {
    pub position: Point,
    pub distance: f64,
    pub proportion_along_side: f64, // how far of the way from left to right we go along the side
    pub side: Side,
}

// allows us to implement Ord based on distance of the rayhit without making rh1 == rh2 depend only on distance (i.e. it remains actual full equality)
pub struct RayHitOrderer {
    pub rh: RayHit,
}

impl PartialEq for RayHitOrderer {
    fn eq(&self, other: &Self) -> bool {
        self.rh.distance == other.rh.distance
    }
}

impl Eq for RayHitOrderer {} // PartialEQ already handles functionality, but must be written out; do not remove

impl PartialOrd for RayHitOrderer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.rh.distance > other.rh.distance {
            Some(Ordering::Greater)
        } else if self.rh.distance < other.rh.distance {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for RayHitOrderer {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.rh.distance > other.rh.distance {
            Ordering::Greater
        } else if self.rh.distance < other.rh.distance {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl RayHitOrderer {
    pub fn new(rayhit: RayHit) -> Self {
        RayHitOrderer { rh: rayhit }
    }
}

// cast a ray and return the ordered list of all hits, ending at the closest wall hit
pub fn raycast(
    game: &Game,
    angle_relative_to_player: f64,
    player_angle: f64,
) -> BinaryHeap<RayHitOrderer> {
    let ray_angle = player_angle + angle_relative_to_player;
    let mut closest_wall_hit: Option<RayHit> = None;
    let mut rayhits_ordered: BinaryHeap<RayHitOrderer> = BinaryHeap::new();

    // find closest wall
    for w in &game.map.wall_sides {
        let intersection: Option<RayHit> = intersect(
            Point {
                x: game.player.position_x,
                y: game.player.position_y,
            },
            ray_angle,
            w.clone(), // TODO remove need for this clone
        );
        if let Some(rayhit) = intersection {
            // didnt hit nothing
            // if its a wall, discard if its not cloesest, otherwise overwrite closest
            if let Some(closest_wall_hit_value) = &closest_wall_hit
                && closest_wall_hit_value.distance < rayhit.distance
            {
                continue;
            }
            closest_wall_hit = Some(rayhit);
        }
    }

    // list all blocks closer than closest wall in order of distance
    for b in &game.map.block_sides {
        let intersection: Option<RayHit> = intersect(
            Point {
                x: game.player.position_x,
                y: game.player.position_y,
            },
            ray_angle,
            b.clone(), // TODO remove need for this clone
        );
        if let Some(rayhit) = intersection {
            // didnt hit nothing
            if let Some(closest_wall_hit_value) = &closest_wall_hit
                && closest_wall_hit_value.distance < rayhit.distance
            {
                continue;
            }
            rayhits_ordered.push(RayHitOrderer { rh: rayhit });
        }
    }

    // TODO separate wall_hit, bottom_block_hits_ top_block hits
    if let Some(wall_hit) = closest_wall_hit {
        rayhits_ordered.push(RayHitOrderer { rh: wall_hit });
    }

    return rayhits_ordered;
}

//checks wether a ray intersect the line between two given points
pub fn intersect(ray_origin: Point, ray_angle: f64, side: Side) -> Option<RayHit> {
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
        distance: distance,
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
