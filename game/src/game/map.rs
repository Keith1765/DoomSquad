use std::ops::{Add, Sub};

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum SideType {
    Wall,
    Block,
}

#[derive(Clone)]
pub struct Side {
    pub point1: Point,
    pub point2: Point,
    pub side_type: SideType,
    pub angle_in_world: f64,
}

#[derive(Clone)]
pub struct Shape {
    pub sides: Vec<Side>,
    pub shape_type: SideType,
}

impl Shape {
    pub fn from_points(points: Vec<Point>, shape_type: SideType) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut sides: Vec<Side> = Vec::new();
        let mut point1: Point;
        let mut point2: Point = *points.last()?;
        for i in 0..points.len() {
            point1 = point2;
            point2 = *points.get(i)?;
            sides.push(Side {
                point1: point1,
                point2: point2,
                side_type: shape_type,
                angle_in_world: 0.0, // TODO actually calculate angle
            });
        }
        Some(Shape {
            sides: sides,
            shape_type: shape_type,
        })
    }
}

// TODO master shape and side list

pub struct Map {
    pub id: usize,
    pub border: Shape, // mainly for topdown renderer (maybe change to rectangle?)
    pub walls: Vec<Shape>,
    pub blocks: Vec<Shape>,
    //pub points_in_border: Vec<Point>,
}

impl Map {
    pub fn new() -> Option<Self> {
        Some(Self {
            id: 0,
            border: Shape::from_points(
                vec![
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
                ],
                SideType::Wall,
            )?,
            walls: vec![Shape::from_points(
                vec![
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
                ],
                SideType::Wall,
            )?],
            blocks: Vec::new(),
            //points_in_border: Vec::new(),
        })
    }
}
