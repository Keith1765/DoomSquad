use std::{
    ops::{Add, Sub},
    rc::Rc,
};

pub const LEVEL_HEIGHT: f64 = 25.0; // TODO different for every map

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
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
pub enum Orientation {
    Top,
    Bottom,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    Wall,
    Block(Orientation),
}

#[derive(Clone, PartialEq)]
pub struct Side {
    pub point1: Point,
    pub point2: Point,
    pub angle_in_world: f64,
    pub shape: Rc<Shape>,
}

impl Side {
    pub fn new(point1: Point, point2: Point, side_type: ShapeType, shape: Rc<Shape>) -> Self {
        return Side {
            point1: point1,
            point2: point2,
            angle_in_world: ((point1.x - point2.x) / (point1.y - point2.y)).atan(),
            shape: shape,
        };
    }
}

// TODO remove necessety for type; justdistinguish by which list it's in

#[derive(Clone, PartialEq)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub height: f64,
}

impl Shape {
    // returns the sides in the shape and the shape itself as tuple
    // add side vector from tuple into side list and shape into shape list
    pub fn from_points(
        points: Vec<Point>,
        shape_type: ShapeType,
        height: f64,
    ) -> Option<(Vec<Side>, Rc<Shape>)> {
        if points.is_empty() {
            return None;
        }
        let shape = Rc::new(Shape {
            shape_type: shape_type,
            height: height,
        });
        let mut sides: Vec<Side> = Vec::new();
        let mut point1: Point;
        let mut point2: Point = *points.last()?;
        for i in 0..points.len() {
            point1 = point2;
            point2 = *points.get(i)?;
            sides.push(Side::new(point1, point2, shape_type, Rc::clone(&shape))); // TODO make height passable to method
        }
        return Some((sides, Rc::clone(&shape)));
    }
}

pub struct Map {
    pub id: usize,
    //pub border: Shape, // mainly for topdown renderer (maybe change to rectangle?)
    pub wall_sides: Vec<Side>,
    pub wall_shapes: Vec<Rc<Shape>>,
    pub block_sides: Vec<Side>,
    pub block_shapes: Vec<Rc<Shape>>,
    //pub points_in_border: Vec<Point>,
}

impl Map {
    pub fn new() -> Option<Self> {
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
        let wall_return: (Vec<Side>, Rc<Shape>) =
            Shape::from_points(wall_points, ShapeType::Wall, LEVEL_HEIGHT)?;

        let bottom_block_points: Vec<Point> = vec![
            Point { x: 200.0, y: 200.0 },
            Point { x: 175.0, y: 200.0 },
            Point { x: 175.0, y: 175.0 },
        ];
        let mut bottom_block_return: (Vec<Side>, Rc<Shape>) = Shape::from_points(
            bottom_block_points,
            ShapeType::Block(Orientation::Bottom),
            10.0,
        )?;

        let top_block_points: Vec<Point> = vec![
            Point { x: 300.0, y: 200.0 },
            Point { x: 250.0, y: 200.0 },
            Point { x: 250.0, y: 175.0 },
        ];
        let mut top_block_return: (Vec<Side>, Rc<Shape>) =
            Shape::from_points(top_block_points, ShapeType::Block(Orientation::Top), 10.0)?;

        let wall_sides: Vec<Side> = wall_return.0;
        let wall_shapes: Vec<Rc<Shape>> = Vec::from([wall_return.1]);
        let mut block_sides: Vec<Side> = Vec::new();
        block_sides.append(&mut bottom_block_return.0);
        block_sides.append(&mut top_block_return.0);
        let block_shapes: Vec<Rc<Shape>> = Vec::from([bottom_block_return.1, top_block_return.1]);

        Some(Self {
            id: 0,
            wall_sides: wall_sides,
            wall_shapes: wall_shapes,
            block_sides: block_sides,
            block_shapes: block_shapes,
            //points_in_border: Vec::new(),
        })
    }
}
