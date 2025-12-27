use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    rc::Rc,
};

use crate::game::{
    Game,
    map::{Orientation, Point, Shape, ShapeID, ShapeType, Side},
};

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

// the 'slice' a ray makes through one block
#[derive(Clone, PartialEq)]
pub struct BlockSlice {
    pub entry_hit: RayHit,
    pub exit_hit: RayHit,
}

// struct BlockSliceOrderer {
//     bs: BlockSlice // does not stand for bullshit
// }

// a slice of the map: the wall at the back, the bottom blocks and the top blocks
#[derive(Clone, PartialEq)]
pub struct MapSlice {
    pub wall_hit: Option<RayHit>,
    pub bottom_block_slices: Vec<BlockSlice>,
    pub top_block_slices: Vec<BlockSlice>,
}

// TODO separate into multiple functions
// TODO also return block were standing on/under in some form
// cast a ray and return the ordered list of all hits, ending at the closest wall hit
pub fn raycast(game: &Game, angle_relative_to_player: f64, player_angle: f64) -> MapSlice {
    let ray_angle = player_angle + angle_relative_to_player;

    // find closest wall
    let mut closest_wall_hit: Option<RayHit> = None;
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
    let mut block_rayhits_ordered: BinaryHeap<RayHitOrderer> = BinaryHeap::new();
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
            block_rayhits_ordered.push(RayHitOrderer { rh: rayhit });
        }
    }

    // we go through the rayhits back to front and remember which block (shape) it belonged to
    // when we find another rayhit for that shape, we've exited the shape and can

    let mut bottom_block_slices: Vec<BlockSlice> = Vec::new();
    let mut top_block_slices: Vec<BlockSlice> = Vec::new();

    let mut bottom_blocks_currently_inside: HashMap<ShapeID, RayHit> = HashMap::new();
    let mut top_blocks_currently_inside: HashMap<ShapeID, RayHit> = HashMap::new();

    while !block_rayhits_ordered.is_empty() {
        if let Some(rh_ordering) = block_rayhits_ordered.pop() {
            let rh = rh_ordering.rh;

            match &rh.side.shape.shape_type {
                ShapeType::Wall => {} // TODO update when closest_wall_hit no longer in rayhits_ordered
                ShapeType::Block(Orientation::Bottom) => {
                    if let Some(shape_exit_hit) =
                        bottom_blocks_currently_inside.remove(&rh.side.shape.id)
                    // if true, we just exited a block we were inside
                    {
                        bottom_block_slices.push(BlockSlice {
                            entry_hit: rh,
                            exit_hit: shape_exit_hit,
                        }); // build the slice of the block
                    } else {
                        bottom_blocks_currently_inside.insert(rh.side.shape.id, rh); // if we werent in that shape already, were inside it now
                    }
                }
                ShapeType::Block(Orientation::Top) => {
                    if let Some(shape_exit_hit) =
                        top_blocks_currently_inside.remove(&rh.side.shape.id)
                    // if true, we just exited a block we were inside
                    {
                        top_block_slices.push(BlockSlice {
                            entry_hit: rh,
                            exit_hit: shape_exit_hit,
                        }); // build the slice of the block
                    } else {
                        top_blocks_currently_inside.insert(rh.side.shape.id, rh); // if we werent in that shape already, were inside it now
                    }
                }
            }
        }
    }

    return MapSlice {
        wall_hit: closest_wall_hit,
        bottom_block_slices,
        top_block_slices,
    };
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
