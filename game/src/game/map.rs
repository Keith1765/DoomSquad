use std::ops::Sub;

#[derive(Clone,Copy)]
pub struct Point {
    pub x : f64,
    pub y : f64,
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

enum SideType {
    Wall,
    Block,
}

pub struct Side {
    point1: Point,
    point2: Point,
    side_type: SideType,
}

//Points are counterclockwise
pub struct Shape {
    pub points: Vec<Point>
}


pub struct Map {
    pub border: Shape,
    pub objects: Vec<Shape>,
}

impl Map {
    pub fn new() -> Self {
        Self { 
            border: Shape{
                points: vec![
                    Point { x: 200.0, y: 100.0 },
Point { x: 250.0, y: 200.0 },
Point { x: 350.0, y: 200.0 },
Point { x: 275.0, y: 250.0 },
Point { x: 300.0, y: 350.0 },
Point { x: 200.0, y: 300.0 },
Point { x: 100.0, y: 350.0 },
Point { x: 125.0, y: 250.0 },
Point { x: 50.0,  y: 200.0 },
Point { x: 150.0, y: 200.0 },


                    ]
            },
            objects: Vec::new(), 
        }
    }

    

    
}