use super::map::Map;
use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    game::{map::{LEVEL_HEIGHT, Point}, movement::Mover},
};
use minifb::{Key, MouseMode, Window};
use std::f64::consts::PI;

const ROTATIONSPEED: f64 = 2.0;
const MOVESPEED: f64 = 1.0;
const FLYUPANDDOWNSPEED: f64 = 1.0;
const PLAYER_VIEW_HEIGHT: f64 = 15.0;
const PLAYER_HEAD_HEIGHT: f64 = 15.0;

// TODO refactor to use entites?
#[derive(Clone)]
pub struct Player {
    pub mover: Mover,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub last_mouse_x: f32,
}

impl Player {
    pub fn new() -> Self {
        let pa: f64 = 4.0;
        Self {
            mover: Mover {
                position: Point { x: 187.5, y: 225.0 },
                floor_level: 0.0,
                foot_level: 0.0,
                view_level: PLAYER_VIEW_HEIGHT,
                height: PLAYER_HEAD_HEIGHT,
                facing_direction: pa, 
            },
            velocity_x: pa.cos() * ROTATIONSPEED,
            velocity_y: pa.sin() * ROTATIONSPEED,
            last_mouse_x: SCREEN_WIDTH as f32 / 2.0,
        }
    }

    pub fn update(&mut self, window: &Window, _map: &Map) {
        if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
            self.check_angle();
            let dx = mx - self.last_mouse_x; // mouse delta
            self.mover.facing_direction += dx as f64 * 0.003; // sensitivity

            self.last_mouse_x = mx; // store for next frame
            self.update_dir();
        }
        if window.is_key_down(Key::Q) {
            self.check_angle();
            self.mover.facing_direction -= 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::E) {
            self.check_angle();
            self.mover.facing_direction += 0.1;
            self.update_dir();
        }

        if window.is_key_down(Key::W) {
            self.mover.position.x += self.velocity_x * MOVESPEED;
            self.mover.position.y += self.velocity_y * MOVESPEED;
        }

        if window.is_key_down(Key::A) {
            self.mover.position.x += self.velocity_y * MOVESPEED;
            self.mover.position.y -= self.velocity_x * MOVESPEED;
        }
        if window.is_key_down(Key::D) {
            self.mover.position.x -= self.velocity_y * MOVESPEED;
            self.mover.position.y += self.velocity_x * MOVESPEED;
        }

        if window.is_key_down(Key::S) {
            self.mover.position.x -= self.velocity_x * MOVESPEED;
            self.mover.position.y -= self.velocity_y * MOVESPEED;
        }

        if window.is_key_down(Key::Space) {
            self.mover.view_level += FLYUPANDDOWNSPEED;
        }

        if window.is_key_down(Key::LeftShift) {
            self.mover.view_level -= FLYUPANDDOWNSPEED;
        }
    }

    fn check_angle(&mut self) {
        if self.mover.facing_direction < 0.1 {
            self.mover.facing_direction += 2.0 * PI
        }
        if self.mover.facing_direction > 2.0 * PI {
            self.mover.facing_direction -= 2.0 * PI
        }
    }

    fn update_dir(&mut self) {
        self.velocity_x = self.mover.facing_direction.cos() * ROTATIONSPEED;
        self.velocity_y = self.mover.facing_direction.sin() * ROTATIONSPEED;
    }
}
