// for some reason used imports were being flagged??? idfk
#[allow(unused_imports)]
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    f64::consts::PI,
    rc::Rc,
};

#[allow(unused_imports)]
use crate::game::{
    Game,
    map::{Map, Point, Shape, ShapeID, ShapeType, Side},
    map_grid::MapGrid,
    movement::Mover,
    player::{self, PLAYER_VIEW_HEIGHT, Player},
};

#[derive(Clone, PartialEq)]
pub struct RayHit {
    pub position: Point,
    pub distance: f64,
    pub proportion_along_side: f64, // how far of the way from left to right we go along the side
    pub side: Rc<Side>,
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
        Some(self.cmp(other))
    }
}

impl Ord for RayHitOrderer {
    fn cmp(&self, other: &Self) -> Ordering {
        // we need some tolerance for floating point impercition, thus te 0.1
        if (self.rh.distance - other.rh.distance) > 0.1 {
            Ordering::Greater
        } else if (self.rh.distance - other.rh.distance) < -0.1 {
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

// a rayhit-made slice of the map: the wall at the back, and all the slices through blocks
#[derive(Clone, PartialEq)]
pub struct MapSlice {
    pub wall_hit: Option<RayHit>,
    pub block_slices: Vec<BlockSlice>,
    pub hits_blocks_currently_inside: Vec<RayHit>,
}

// cast a ray and return the ordered list of all hits, ending at the closest wall hit
pub fn raycast(map: &Map, angle_relative_to_player: f64, player: &Player) -> MapSlice {
    let ray_angle = player.mover.facing_direction + angle_relative_to_player;

    // find closest wall
    let mut closest_wall_hit: Option<RayHit> = None;
    for wall in &map.wall_sides {
        let rayhit: Option<RayHit> = intersect(
            Point {
                x: player.mover.position.x,
                y: player.mover.position.y,
            },
            ray_angle,
            Rc::clone(wall),
        );
        if let Some(rayhit) = rayhit {
            // didnt hit nothing
            // if its a wall, discard if its not closest, otherwise overwrite closest
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
    for block in &map.block_sides {
        let rayhit: Option<RayHit> = intersect(
            Point {
                x: player.mover.position.x,
                y: player.mover.position.y,
            },
            ray_angle,
            Rc::clone(block),
        );
        if let Some(rayhit) = rayhit {
            // didnt hit nothing
            if let Some(closest_wall_hit_value) = &closest_wall_hit
                && closest_wall_hit_value.distance < rayhit.distance
            {
                continue;
            }
            block_rayhits_ordered.push(RayHitOrderer { rh: rayhit });
        }
    }

    // we have now established an ordered list of rayhits
    // we go through the rayhits back to front and remember which block (shape) it belonged to
    // when we find another rayhit for that shape, we've exited the shape and can
    // put int inot a fully-made block_slice
    let mut block_slices: Vec<BlockSlice> = Vec::new();

    // block which the ray is currently passing through (in 2dim space), in its imagined backtrack through its own path
    let mut blocks_currently_over: HashMap<ShapeID, RayHit> = HashMap::new();

    while !block_rayhits_ordered.is_empty() {
        if let Some(rh_ordering) = block_rayhits_ordered.pop() {
            let rh = rh_ordering.rh;

            if let Some(shape_exit_hit) = blocks_currently_over.remove(&rh.side.shape.id)
            // if true, we just exited a block we were  with our ray backtrack
            {
                block_slices.push(BlockSlice {
                    entry_hit: rh,
                    exit_hit: shape_exit_hit,
                }); // build the slice of the block
            } else {
                blocks_currently_over.insert(rh.side.shape.id, rh); // if we werent in that shape already, were inside it now
            }
        }
    }

    // finish and return the whole slice of the whole map
    MapSlice {
        wall_hit: closest_wall_hit,
        block_slices,
        hits_blocks_currently_inside: blocks_currently_over.into_values().collect(),
    }
}

/// checks wether a ray intersect the line between two given points
pub fn intersect(ray_origin: Point, ray_angle: f64, side: Rc<Side>) -> Option<RayHit> {
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
    Some(RayHit {
        position,
        distance,
        proportion_along_side: proportion,
        side: Rc::clone(&side),
    })
}

/// rotates using roation matrix
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

#[test]
fn test_zero_rotation_leaves_point_unchanged() {
    let point = Point { x: 3.0, y: 4.0 };
    let result = rotate_point_around_origin(point, 0.0);
    assert!((result.x - 3.0).abs() < 0.1);
    assert!((result.y - 4.0).abs() < 0.1);
}

#[test]
fn test_rotate_45_degrees() {
    let point = Point { x: 1.0, y: 0.0 };
    let result = rotate_point_around_origin(point, PI / 4.0);
    let expected = 1.0 / 2.0_f64.sqrt();
    assert!((result.x - expected).abs() < 0.1);
    assert!((result.y - expected).abs() < 0.1);
}

#[test]
fn test_rotate_negative_90_degrees() {
    let point = Point { x: 1.0, y: 0.0 };
    let result = rotate_point_around_origin(point, -PI / 2.0);
    assert!((result.x - 0.0).abs() < 0.1);
    assert!((result.y - -1.0).abs() < 0.1);
}

#[test]
fn test_rotate_360_degrees_returns_original() {
    let point = Point { x: 3.0, y: 4.0 };
    let result = rotate_point_around_origin(point, 2.0 * PI);
    assert!((result.x - point.x).abs() < 0.1);
    assert!((result.y - point.y).abs() < 0.1);
}

#[test]
fn test_distance_from_origin_preserved() {
    let point = Point { x: 3.0, y: 4.0 };
    let original_dist = (point.x.powi(2) + point.y.powi(2)).sqrt();
    let result = rotate_point_around_origin(point, PI / 7.0);
    let rotated_dist = (result.x.powi(2) + result.y.powi(2)).sqrt();
    assert!((original_dist - rotated_dist).abs() < 0.1);
}

#[test]
fn test_successive_rotations_are_additive() {
    let point = Point { x: 3.0, y: 4.0 };
    let once = rotate_point_around_origin(point, PI / 4.0);
    let twice = rotate_point_around_origin(once, PI / 4.0);
    let combined = rotate_point_around_origin(point, PI / 2.0);
    assert!((twice.x - combined.x).abs() < 0.1);
    assert!((twice.y - combined.y).abs() < 0.1);
}

#[test]
fn test_intersect_basic_hit() {
    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 0.0, y: 0.0 };
    let point1 = Point { x: 5.0, y: 2.0 };
    let point2 = Point { x: 5.0, y: -2.0 };
    let side_in_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, 0.0, side_in_ray);

    assert!(rh.is_some());
    assert!((rh.unwrap().distance - 5.0).abs() < 0.1);
}

#[test]
fn test_intersect_basic_no_hit() {
    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 0.0, y: 0.0 };
    let point1 = Point { x: 5.0, y: 2.0 };
    let point2 = Point { x: 5.0, y: 4.0 };
    let side_not_in_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, 0.0, side_not_in_ray);

    assert!(rh.is_none());
}

#[test]
fn test_intersect_basic_behind_ray() {
    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 0.0, y: 0.0 };
    let point1 = Point { x: -5.0, y: 2.0 };
    let point2 = Point { x: -5.0, y: -2.0 };
    let side_behind_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, 0.0, side_behind_ray);

    assert!(rh.is_none());
}

#[test]
fn test_intersect_angled_offset_hit() {
    // difference to above: player not at origin, ray angled at 45 degrees

    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 5.0, y: -2.0 };
    let point1 = Point { x: 5.0, y: 0.0 };
    let point2 = Point { x: 15.0, y: 0.0 };
    let side_in_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, PI / 4.0, side_in_ray);

    assert!(rh.is_some());
    assert!((rh.unwrap().distance - 2.8).abs() < 0.5);
}

#[test]
fn test_intersect_angled_offset_no_hit() {
    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 5.0, y: -2.0 };
    let point1 = Point { x: 15.0, y: 0.0 };
    let point2 = Point { x: 25.0, y: 0.0 };
    let side_in_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, 0.0, side_in_ray);

    assert!(rh.is_none());
}

#[test]
fn test_intersect_angled_offset_behind_ray() {
    let placeholder_shape = Shape {
        id: 0,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    };

    let point0 = Point { x: 5.0, y: -2.0 };
    let point1 = Point { x: 5.0, y: -4.0 };
    let point2 = Point { x: 15.0, y: -4.0 };
    let side_in_ray = Rc::new(Side::new(0, point1, point2, Rc::new(placeholder_shape), 0));

    let rh = intersect(point0, 0.0, side_in_ray);

    assert!(rh.is_none());
}

#[test]
fn test_raycast() {
    let placeholder_shape = Rc::new(Shape {
        id: 0,
        shape_type: ShapeType::Block,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    });

    let placeholder_wall_shape = Rc::new(Shape {
        id: 1,
        shape_type: ShapeType::Wall,
        bottom: 0.0,
        height: 5.0,
        color: 0x000000,
        surface_color: 0x000000,
    });

    let point0 = Point { x: 0.0, y: 0.0 };
    let point1a = Point { x: 5.0, y: 2.0 };
    let point2a = Point { x: 5.0, y: -2.0 };
    let point1b = Point { x: 5.0, y: 2.0 };
    let point2b = Point { x: 5.0, y: -2.0 };
    let point1c = Point { x: 5.0, y: 2.0 };
    let point2c = Point { x: 5.0, y: -2.0 };
    let side_in_ray_a = Rc::new(Side::new(
        0,
        point1a,
        point2a,
        Rc::clone(&placeholder_shape),
        0,
    ));
    let side_in_ray_b = Rc::new(Side::new(
        0,
        point1b,
        point2b,
        Rc::clone(&placeholder_shape),
        0,
    ));
    let side_in_ray_c = Rc::new(Side::new(
        0,
        point1c,
        point2c,
        Rc::clone(&placeholder_wall_shape),
        0,
    ));

    let map = Map {
        id: 0,
        wall_sides: vec![side_in_ray_c],
        block_sides: vec![side_in_ray_a, side_in_ray_b],
        wall_shapes: vec![placeholder_wall_shape],
        block_shapes: vec![placeholder_shape],
        side_count: 3,
        shape_count: 2,
    };

    let placeholder_player = Player {
        mover: Mover {
            position: point0,
            floor_level: 0.0,
            foot_level: 0.0,
            view_level: PLAYER_VIEW_HEIGHT,
            height: PLAYER_VIEW_HEIGHT,
            facing_direction: 0.0,
        },
        velocity_x: 0.0,
        velocity_y: 0.0,
        last_mouse_x: 0.0,
        last_mouse_y: 0.0,
        godmode: false,
        move_speed: 0.0,
        is_sliding: false,
        last_input: player::LastInputDirection::A,
        slide_cooldown: 0,
        is_jumping: false,
        vertical_velocity: 0.0,
        gravity: 0.0,
        rocketlauncher_cooldown: 0,
        hp: 0.0,
        arrow_cooldown: 0,
        bullet_cooldown: 0,
        size: 0.0,
        using_rocketlauncher: false,
        interacting: false,
        jumping_allowed: false,
        jumping_allowed_timer: 0,
        vertcal_aim: 0.0,
        aim_mode: true,
    };

    let map_slice = raycast(&map, 0.0, &placeholder_player);

    assert!(map_slice.wall_hit.is_some());
    assert!(map_slice.block_slices.len() == 1);
}
