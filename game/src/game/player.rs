use minifb::{Key, Window};
use std::f64::consts::PI;

use super::map::Map;

const ROTATIONSPEED: f64 = 3.0;
const MOVESPEED: f64 = 2.0;

#[derive(Clone, Copy)]
pub struct Player {
    pub px:f64,
    pub py:f64,
    pub pdx:f64,
    pub pdy:f64,
    pub pa:f64,
}

//TODO remove pdx and pdy
//TODO long variable names

impl Player {
    pub fn new() -> Self {
        let pa:f64=-PI/2.0;
        Self {
            px : 300.0,
            py : 300.0,
            pdx : pa.cos()*ROTATIONSPEED,
            pdy : pa.sin()*ROTATIONSPEED,
            pa ,
        }
    }

    pub fn update(&mut self, window: &Window, map: &Map) {
        if window.is_key_down(Key::A) {
            self.check_angle();
            self.pa -= 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::D) {
            self.check_angle();
            self.pa += 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::W){
            self.px += self.pdx * MOVESPEED;
            self.py += self.pdy * MOVESPEED;
        }

        if window.is_key_down(Key::S){
            self.px -= self.pdx * MOVESPEED;
            self.py -= self.pdy * MOVESPEED;
        }

    }
    
    fn check_angle (&mut self) {
        if self.pa <0.1 {self.pa += 2.0*PI}
        if self.pa > 2.0*PI {self.pa -= 2.0*PI}
    }

    fn update_dir(&mut self) {
        self.pdx = self.pa.cos()*ROTATIONSPEED;
        self.pdy = self.pa.sin()*ROTATIONSPEED;
    }
    
}