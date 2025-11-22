pub type Point = (f64,f64);

enum SideType {
    Wall,
    Block,
}

pub struct Side {
    point1: Point,
    point2: Point,
    side_type: SideType,
}

pub struct Shape {
    points: Vec<Point>
}


pub struct Map {
    pub boarder: Shape,
    pub objects: Vec<Shape>,
}

impl Map {
    pub fn new() -> Self {
        Self { 
            boarder: Shape{
                points: vec![
                    (100.0,100.0),
                    (300.0,100.0),
                    (300.0,300.0),
                    (100.0,300.0),
                    ]
            },
            objects: Vec::new(), 
        }
    }

    

    
}