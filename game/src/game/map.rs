use std::{
    f64::consts::PI,
    hash::{Hash, Hasher},
    ops::{Add, Sub},
    rc::Rc,
};

use crate::game::entities::Entity;
use crate::render::sprites::Sprite;

pub const LEVEL_HEIGHT: f64 = 25.0; // TODO different for every map

pub type ShapeID = usize;
type SideID = usize;

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    // TODO tests for this function
    pub fn distance_to(self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        (dx.powf(2.0) + dy.powf(2.0)).sqrt()
    }

    // TODO tests for this function
    pub fn angle_to(self, other: &Self) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;

        let mut angle = (dy / dx).atan();

        // without this atan only works for 180 degrees
        if dx < 0.0 {
            angle += PI;
        }

        // if angle is negative, rotate it around 360deg to get same angle expressed positively
        if angle < 0.0 {
            angle += 2.0 * PI;
        }

        angle
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    Wall,  // walls are ray-terminating
    Block, // blocks are not
}

#[derive(Clone, PartialEq)]
pub struct Side {
    pub id: SideID,
    pub point1: Point,
    pub point2: Point,
    pub angle_in_world: f64,
    pub length: f64,
    pub shape: Rc<Shape>,
    pub texture_id: usize,
}

impl Side {
    pub fn new(
        id: usize,
        point1: Point,
        point2: Point,
        shape: Rc<Shape>,
        texture_id: usize,
    ) -> Self {
        return Side {
            id: id,
            point1: point1,
            point2: point2,
            angle_in_world: ((point1.x - point2.x) / (point1.y - point2.y)).atan(),
            length: point1.distance_to(&point2),
            shape: shape,
            texture_id: texture_id,
        };
    }
}

#[derive(Clone)]
pub struct Shape {
    pub id: ShapeID,
    pub shape_type: ShapeType,
    pub bottom: f64,
    pub height: f64,
    pub color: u32,         // TODO remove when no longer needed
    pub surface_color: u32, // will be ignored for walls
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Shape {}
impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub struct Map {
    pub id: usize,
    //pub border: Shape, // mainly for topdown renderer (maybe change to rectangle?)
    pub wall_sides: Vec<Side>,
    pub wall_shapes: Vec<Rc<Shape>>,
    pub block_sides: Vec<Side>,
    pub block_shapes: Vec<Rc<Shape>>, //TODO are the shape vectors even needed?
    pub entities: Vec<Entity>,
    side_count: usize,
    shape_count: usize,
}

impl Map {
    pub fn new() -> Option<Self> {
        let mut map = Self {
            id: 0,
            wall_sides: Vec::new(),
            wall_shapes: Vec::new(),
            block_sides: Vec::new(),
            block_shapes: Vec::new(),
            entities: Vec::new(),
            side_count: 0,
            shape_count: 0,
            //points_in_border: Vec::new(),
        };
        let wall_points: Vec<Point> = vec![
            Point { x: 200.0, y: 100.0 },
            Point { x: 250.0, y: 200.0 },
            Point { x: 350.0, y: 200.0 },
            Point { x: 275.0, y: 250.0 },
            Point { x: 300.0, y: 350.0 },
            Point { x: 200.0, y: 300.0 },
            Point { x: 100.0, y: 350.0 },
            Point { x: 125.0, y: 250.0 },
            Point { x: 50.0, y: 200.0 },
            Point { x: 150.0, y: 200.0 },
        ];
        map.add_shape_from_points(
            wall_points.clone(), // TODO remove this clone(), also the others
            ShapeType::Wall,
            0.0,
            LEVEL_HEIGHT,
            0x00ff00,
            0xffff00,
            vec![0; wall_points.len()],
        )?;

        let bottom_block_points: Vec<Point> = vec![
            Point { x: 200.0, y: 200.0 },
            Point { x: 175.0, y: 200.0 },
            Point { x: 175.0, y: 175.0 },
        ];
        map.add_shape_from_points(
            bottom_block_points.clone(),
            ShapeType::Block,
            0.0,
            10.0,
            0x0000ff,
            0xffff00,
            vec![0; bottom_block_points.len()],
        )?;

        let bottom_block_points_2: Vec<Point> = vec![
            Point { x: 200.0, y: 215.0 },
            Point { x: 175.0, y: 215.0 },
            Point { x: 175.0, y: 200.0 },
            Point { x: 185.0, y: 200.0 },
        ];
        map.add_shape_from_points(
            bottom_block_points_2.clone(),
            ShapeType::Block,
            0.0,
            5.0,
            0x0000ff,
            0xffff00,
            vec![0; bottom_block_points_2.len()],
        )?;

        let top_block_points: Vec<Point> = vec![
            // Point { x: 300.0, y: 225.0 },
            // Point { x: 250.0, y: 225.0 },
            // Point { x: 250.0, y: 200.0 },
            Point { x: 205.0, y: 205.0 },
            Point { x: 180.0, y: 205.0 },
            Point { x: 180.0, y: 178.0 },
        ];
        map.add_shape_from_points(
            top_block_points.clone(),
            ShapeType::Block,
            15.0,
            10.0,
            0xff0000,
            0xffff00,
            vec![0; top_block_points.len()],
        )?;

        let small_block_points: Vec<Point> = vec![
            Point { x: 200.0, y: 200.0 },
            Point { x: 190.0, y: 200.0 },
            Point { x: 190.0, y: 190.0 },
        ];
        map.add_shape_from_points(
            small_block_points.clone(),
            ShapeType::Block,
            10.0,
            1.0,
            0xffffff,
            0xff00ff,
            vec![0; small_block_points.len()],
        )?;

        let test_entity = Entity {
            position: Point { x: 230.0, y: 210.0 },
            vertical_position: 0.0,
            facing_angle: 0.0,
            sprite: Sprite {
                color: 0xff00ff,
                height: 15.0,
                width: 15.0,
            },
        };
        map.entities.push(test_entity);

        Some(map)
    }

    // returns the sides in the shape and the shape itself as tuple
    // add side vector from tuple into side list and shape into shape list
    pub fn add_shape_from_points(
        &mut self,
        points: Vec<Point>,
        shape_type: ShapeType,
        bottom: f64,
        height: f64,
        color: u32, // TODO remove when no longer needed
        surface_color: u32,
        texture_ids: Vec<usize>, // TODO change later so that different sides can have different textures
    ) -> Option<()> {
        if points.is_empty() || points.len() != texture_ids.len() {
            return None;
        }
        let shape = Rc::new(Shape {
            id: self.shape_count,
            shape_type: shape_type,
            bottom: bottom,
            height: height,
            color: color,
            surface_color,
        });

        // references to push to the corect list
        let sides: &mut Vec<Side> = match shape_type {
            ShapeType::Wall => &mut self.wall_sides,
            ShapeType::Block => &mut self.block_sides,
        };
        let shapes: &mut Vec<Rc<Shape>> = match shape_type {
            ShapeType::Wall => &mut self.wall_shapes,
            ShapeType::Block => &mut self.block_shapes,
        };

        let mut point1: Point;
        let mut point2: Point = *points.last()?;
        for i in 0..points.len() {
            point1 = point2;
            point2 = *points.get(i)?;
            if let Some(texture_id) = texture_ids.get(i) {
                sides.push(Side::new(
                    self.side_count,
                    point1,
                    point2,
                    Rc::clone(&shape),
                    *texture_id,
                ));
            }
            
            self.side_count += 1;
        }
        shapes.push(shape);
        self.shape_count += 1;
        return Some(());
    }
}

#[test]
fn test_angle() {
    let p1 = Point { x: 0.0, y: 0.0 };

    let p2 = Point { x: 10.0, y: 0.0 };
    assert!((p1.angle_to(&p2) - 0.0).abs() < 0.1);

    let p3 = Point { x: -10.0, y: 0.0 };
    assert!((p1.angle_to(&p3) - PI).abs() < 0.1);

    let p4 = Point { x: 0.0, y: 10.0 };
    assert!((p1.angle_to(&p4) - PI / 2.0).abs() < 0.1);

    let p5 = Point { x: 0.0, y: -10.0 };
    assert!((p1.angle_to(&p5) - 3.0 * (PI / 2.0)).abs() < 0.1);

    let p6 = Point { x: 10.0, y: 10.0 };
    assert!((p1.angle_to(&p6) - PI / 4.0).abs() < 0.1);

    let p7 = Point { x: -10.0, y: 10.0 };
    assert!((p1.angle_to(&p7) - 3.0 * (PI / 4.0)).abs() < 0.1);

    let p8 = Point { x: -10.0, y: -10.0 };
    assert!((p1.angle_to(&p8) - 5.0 * (PI / 4.0)).abs() < 0.1);

    let p9 = Point { x: 10.0, y: -10.0 };
    assert!((p1.angle_to(&p9) - 7.0 * (PI / 4.0)).abs() < 0.1);
}
