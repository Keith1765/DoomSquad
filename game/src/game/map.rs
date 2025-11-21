pub const MAPX: usize = 8;
pub const MAPY: usize = 8;
pub const MAPSIZE: usize = 64;

pub struct Map {
    pub data : [usize; 64],
}

impl Map {
    pub fn new() -> Self {
        Self { 
            data: [
                1,1,1,1,1,1,1,1,
                1,0,1,0,0,0,0,1,
                1,0,1,0,0,0,0,1,
                1,0,1,0,0,0,0,1,
                1,0,1,0,0,1,0,1,
                1,0,0,0,0,1,0,1,
                1,0,0,0,0,0,0,1,
                1,1,1,1,1,1,1,1
    ],
        }
        
    }
}