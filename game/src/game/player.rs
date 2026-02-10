use super::map::Map;
use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    game::map::{LEVEL_HEIGHT, Point},
};
use minifb::{Key, MouseMode, Window};
use std::f64::consts::PI;

const ROTATIONSPEED: f64 = 2.0;
const MOVESPEED: f64 = 1.0;
const FLYUPANDDOWNSPEED: f64 = 1.0;

// TODO refactor to use entites?
#[derive(Clone, Copy)]
pub struct Player {
    pub position: Point,
    pub view_height: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub view_angle: f64,
    pub last_mouse_x: f32,
}

impl Player {
    pub fn new() -> Self {
        let pa: f64 = 4.0;
        Self {
            position: Point { x: 187.5, y: 225.0 },
            view_height: 0.0,
            velocity_x: pa.cos() * ROTATIONSPEED,
            velocity_y: pa.sin() * ROTATIONSPEED,
            view_angle: pa,
            last_mouse_x: SCREEN_WIDTH as f32 / 2.0,
        }
    }

    pub fn update(&mut self, window: &Window, _map: &Map) {
        if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
            self.check_angle();
            let dx = mx - self.last_mouse_x; // mouse delta
            self.view_angle += dx as f64 * 0.003; // sensitivity

            self.last_mouse_x = mx; // store for next frame
            self.update_dir();
        }
        if window.is_key_down(Key::Q) {
            self.check_angle();
            self.view_angle -= 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::E) {
            self.check_angle();
            self.view_angle += 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::W) {
            self.position.x += self.velocity_x * MOVESPEED;
            self.position.y += self.velocity_y * MOVESPEED;
        }

        if window.is_key_down(Key::A) {
            self.position.x += self.velocity_y * MOVESPEED;
            self.position.y -= self.velocity_x * MOVESPEED;
        }
        if window.is_key_down(Key::D) {
            self.position.x -= self.velocity_y * MOVESPEED;
            self.position.y += self.velocity_x * MOVESPEED;
        }

        if window.is_key_down(Key::S) {
            self.position.x -= self.velocity_x * MOVESPEED;
            self.position.y -= self.velocity_y * MOVESPEED;
        }

        if window.is_key_down(Key::Space) {
            self.view_height += FLYUPANDDOWNSPEED;
        }

        if window.is_key_down(Key::LeftShift) {
            self.view_height -= FLYUPANDDOWNSPEED;
        }
    }

    fn check_angle(&mut self) {
        if self.view_angle < 0.1 {
            self.view_angle += 2.0 * PI
        }
        if self.view_angle > 2.0 * PI {
            self.view_angle -= 2.0 * PI
        }
    }

    fn update_dir(&mut self) {
        self.velocity_x = self.view_angle.cos() * ROTATIONSPEED;
        self.velocity_y = self.view_angle.sin() * ROTATIONSPEED;
    }
}
