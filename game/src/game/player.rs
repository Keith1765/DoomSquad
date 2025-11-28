use minifb::{Key, MouseMode, Window};
use std::f64::consts::PI;
use crate::{HEIGHT, WIDTH};
use super::map::Map;

const ROTATIONSPEED: f64 = 2.0;
const MOVESPEED: f64 = 2.0;

#[derive(Clone, Copy)]
pub struct Player {
    pub px:f64,
    pub py:f64,
    pub pdx:f64,
    pub pdy:f64,
    pub pa:f64,
    pub last_mouse_x: f32,
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
            last_mouse_x: WIDTH as f32 / 2.0,
        }
    }

    pub fn update(&mut self, window: &Window, map: &Map) {
        if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
            self.check_angle();
            let dx = mx - self.last_mouse_x;        // mouse delta
            self.pa += dx as f64 * 0.003; // sensitivity

            self.last_mouse_x = mx; // store for next frame
            self.update_dir();
            
    }
        if window.is_key_down(Key::Q) {
            self.check_angle();
            self.pa -= 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::E) {
            self.check_angle();
            self.pa += 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::W){
            self.px += self.pdx * MOVESPEED;
            self.py += self.pdy * MOVESPEED;
        }

        if window.is_key_down(Key::A){
            self.px += self.pdy * MOVESPEED;
            self.py -= self.pdx * MOVESPEED;
        }
        if window.is_key_down(Key::D){
            self.px -= self.pdy * MOVESPEED;
            self.py += self.pdx * MOVESPEED;
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