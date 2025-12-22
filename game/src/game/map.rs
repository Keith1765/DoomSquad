use std::{
    hash::{Hash, Hasher}, ops::{Add, Sub}, rc::Rc
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
    pub id: usize,
    pub point1: Point,
    pub point2: Point,
    pub angle_in_world: f64,
    pub shape: Rc<Shape>,
}

impl Side {
    pub fn new(id: usize, point1: Point, point2: Point, shape: Rc<Shape>) -> Self {
        return Side {
            id: id,
            point1: point1,
            point2: point2,
            angle_in_world: ((point1.x - point2.x) / (point1.y - point2.y)).atan(),
            shape: shape,
        };
    }
}

// TODO remove necessety for type; justdistinguish by which list it's in

#[derive(Clone)]
pub struct Shape {
    pub id: usize,
    pub shape_type: ShapeType,
    pub height: f64,
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
        map.add_shape_from_points(wall_points, ShapeType::Wall, LEVEL_HEIGHT)?;

        let bottom_block_points: Vec<Point> = vec![
            Point { x: 200.0, y: 200.0 },
            Point { x: 175.0, y: 200.0 },
            Point { x: 175.0, y: 175.0 },
        ];
        map.add_shape_from_points(
            bottom_block_points,
            ShapeType::Block(Orientation::Bottom),
            10.0,
        )?;

        let top_block_points: Vec<Point> = vec![
            // Point { x: 300.0, y: 225.0 },
            // Point { x: 250.0, y: 225.0 },
            // Point { x: 250.0, y: 200.0 },
            Point { x: 205.0, y: 205.0 },
            Point { x: 180.0, y: 205.0 },
            Point { x: 180.0, y: 178.0 },
        ];
        map.add_shape_from_points(top_block_points, ShapeType::Block(Orientation::Top), 10.0)?;

        Some(map)
    }

    // returns the sides in the shape and the shape itself as tuple
    // add side vector from tuple into side list and shape into shape list
    pub fn add_shape_from_points(
        &mut self,
        points: Vec<Point>,
        shape_type: ShapeType,
        height: f64,
    ) -> Option<()> {
        if points.is_empty() {
            return None;
        }
        let shape = Rc::new(Shape {
            id: self.shape_count,
            shape_type: shape_type,
            height: height,
        });

        // references to push to the corect list
        let sides: &mut Vec<Side> = match shape_type {
            ShapeType::Wall => &mut self.wall_sides,
            ShapeType::Block(_) => &mut self.block_sides
        };
        let shapes: &mut Vec<Rc<Shape>> = match shape_type {
            ShapeType::Wall => &mut self.wall_shapes,
            ShapeType::Block(_) => &mut self.block_shapes
        };

        let mut point1: Point;
        let mut point2: Point = *points.last()?;
        for i in 0..points.len() {
            point1 = point2;
            point2 = *points.get(i)?;
            sides.push(Side::new(self.side_count, point1, point2, Rc::clone(&shape)));
            self.side_count+=1;
        }
        shapes.push(shape);
        self.shape_count+=1;
        return Some(());
    }
}
