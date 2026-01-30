use crate::game::map::Point;


#[derive(Clone)]
pub struct Mover {
    pub position: Point,
    pub floor_level: f64,
    pub foot_level: f64,
    pub view_level: f64,
    pub height: f64,
    pub facing_direction: f64,
}