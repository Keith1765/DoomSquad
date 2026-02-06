use super::map::Map;
use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    game::{
        map::{LEVEL_HEIGHT, Point},
        movement::Mover,
    },
};
use minifb::{Key, MouseMode, Window};
use std::f64::consts::PI;

const ROTATION_SPEED: f64 = 2.0;
pub const MOVE_SPEED: f64 = 1.0;
const FLY_UP_DOWN_SPEED: f64 = 1.0;
const MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
pub const MAX_STEP_UP_HEIGHT: f64 = 5.0;
const PLAYER_HEAD_HEIGHT: f64 = 15.0;
pub const PLAYER_VIEW_HEIGHT: f64 = 15.0;

#[derive(Clone)]
pub struct Player {
    pub mover: Mover,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub last_mouse_x: f32,
    pub godmode: bool,
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
            velocity_x: pa.cos() * ROTATION_SPEED,
            velocity_y: pa.sin() * ROTATION_SPEED,
            last_mouse_x: SCREEN_WIDTH as f32 / 2.0,
            godmode: false, // allows flying up and down, no collision (when those are implemented)
        }
    }

    pub fn update(&mut self, window: &Window, map: &Map) {
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
            self.mover.step(MOVE_SPEED, 0.0, map, self.godmode);
        }

        if window.is_key_down(Key::A) {
            self.mover.step(MOVE_SPEED, -PI / 2.0, map, self.godmode);
        }
        if window.is_key_down(Key::D) {
            self.mover.step(MOVE_SPEED, PI / 2.0, map, self.godmode);
        }

        if window.is_key_down(Key::S) {
            self.mover.step(MOVE_SPEED, PI, map, self.godmode);
        }

        if window.is_key_down(Key::Space) && self.godmode {
            self.mover.foot_level += FLY_UP_DOWN_SPEED;
        }

        if window.is_key_down(Key::LeftShift) && self.godmode {
            self.mover.foot_level -= FLY_UP_DOWN_SPEED;
        }

        // TODO make not fiddly (currently switches every tick)
        if window.is_key_down(Key::G) {
            self.godmode = !self.godmode;
        }

        //adjust feet_level to fit floor_level
        // smoothing: only "catch up" foot level with floor level at s smooting speed
        if !self.godmode {
            if (self.mover.foot_level - self.mover.floor_level).abs() < MOVEMENT_SMOOTHING_SPEED {
                self.mover.foot_level = self.mover.floor_level;
            } else if self.mover.foot_level < self.mover.floor_level {
                self.mover.foot_level += MOVEMENT_SMOOTHING_SPEED;
            } else {
                self.mover.foot_level -= MOVEMENT_SMOOTHING_SPEED;
            }
        }
        self.mover.view_level = self.mover.foot_level + PLAYER_VIEW_HEIGHT;
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
        self.velocity_x = self.mover.facing_direction.cos() * ROTATION_SPEED;
        self.velocity_y = self.mover.facing_direction.sin() * ROTATION_SPEED;
    }
}
