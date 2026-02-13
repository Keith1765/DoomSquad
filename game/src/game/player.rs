use super::map::Map;
use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    game::{
        map::{LEVEL_HEIGHT, Point},
        movement::Mover, player,
    },
};
use minifb::{Key, KeyRepeat, MouseMode, Window};
use std::f64::consts::PI;
use crate::game::player::LastInputDirection::*;

const ROTATION_SPEED_MOUSE: f64 = 2.0;
const ROTATION_SPEED_KEYS: f64 = 0.15;
pub const MOVE_SPEED: f64 = 1.5;
const FLY_UP_DOWN_SPEED: f64 = 1.0;
const MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
pub const MAX_STEP_UP_HEIGHT: f64 = 5.0;
const PLAYER_HEAD_HEIGHT: f64 = 15.0;
pub const PLAYER_VIEW_HEIGHT: f64 = 15.0;
const SPRINT_SPEED: f64 = 3.5;
const CROUCH_Distance: f64 = 7.5; 
const SLIDE_COOLDOWN_TIME: i32 = 15;
const STRAIFING_SPEED: f64 = 0.02;

#[derive(Clone,PartialEq, Eq)]
pub enum LastInputDirection {
    W,
    A,
    S,
    D,
    No,
}
#[derive(Clone)]
pub struct Player {
    pub mover: Mover,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub last_mouse_x: f32,
    pub godmode: bool,
    pub move_speed: f64,
    pub is_sliding: bool,
    pub last_input: LastInputDirection,
    pub slice_cooldown: i32,
    pub is_jumping: bool,
    
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
            velocity_x: pa.cos() * ROTATION_SPEED_MOUSE,
            velocity_y: pa.sin() * ROTATION_SPEED_MOUSE,
            last_mouse_x: SCREEN_WIDTH as f32 / 2.0,
            godmode: false, // allows flying up and down, no collision (when those are implemented)
            move_speed: 1.0,
            is_sliding: false,
            last_input: No,
            slice_cooldown: 0,
            is_jumping: false,
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

        if window.is_key_down(Key::Left) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding {
                true => STRAIFING_SPEED,
                false => ROTATION_SPEED_KEYS,
            };

            self.check_angle();
            self.mover.facing_direction -= rotation_factor;
            self.update_dir();
        }

        if window.is_key_down(Key::Right) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding {
                true => STRAIFING_SPEED,
                false => ROTATION_SPEED_KEYS,
            };
            self.check_angle();
            self.mover.facing_direction += rotation_factor;
            self.update_dir();
        }

        if window.is_key_down(Key::W) && ((!self.is_sliding && !self.is_jumping) || self.last_input==W) {
            self.mover.step(self.move_speed, 0.0, map, self.godmode);
            
        }
        if window.is_key_down(Key::A) && ((!self.is_sliding && !self.is_jumping)|| self.last_input==A) {
            self.mover.step(self.move_speed, -PI / 2.0, map, self.godmode);
        }
        if window.is_key_down(Key::D) && ((!self.is_sliding && !self.is_jumping) || self.last_input==D){
            self.mover.step(self.move_speed, PI / 2.0, map, self.godmode);
        }

        if window.is_key_down(Key::S) && ((!self.is_sliding && !self.is_jumping) || self.last_input==S) {
            self.mover.step(self.move_speed, PI, map, self.godmode);
        }

        if window.is_key_down(Key::Space) && self.godmode {
            self.mover.foot_level += FLY_UP_DOWN_SPEED;
        }

        if window.is_key_down(Key::LeftShift) && self.godmode {
            self.mover.foot_level -= FLY_UP_DOWN_SPEED;
        }

        //slowdown movespeed if not sprinting and sliding anymore
        if !window.is_key_down(Key::LeftShift) && !window.is_key_down(Key::Down){
            self.move_speed=1.0;
        }

        //implementing Sprint that gradually increases movement speed
        if window.is_key_down(Key::LeftShift) && !self.godmode{
            if self.move_speed < SPRINT_SPEED-0.1 {
                self.move_speed += 0.1;
            }
            if self.move_speed <SPRINT_SPEED {
                self.move_speed = SPRINT_SPEED;
            }

            if self.move_speed > SPRINT_SPEED && !self.is_sliding {
                self.move_speed = SPRINT_SPEED;
            }
        }

        if (self.slice_cooldown > 0) && !window.is_key_down(Key::Down) {
            self.slice_cooldown -= 1;
        } 
        //init slide gives speed boost
        if window.is_key_down(Key::Down) && !self.is_sliding && !self.godmode && self.slice_cooldown == 0{
            if self.move_speed > 1.5 {
                self.move_speed += 5.0;
                self.is_sliding = true;
                self.mover.foot_level -=CROUCH_Distance;
                if window.is_key_down(Key::D) {self.last_input = D};
                if window.is_key_down(Key::A) {self.last_input = A};
                if window.is_key_down(Key::S) {self.last_input = S};
                if window.is_key_down(Key::W) {self.last_input = W};
            }
            
        }
        
        //during slide speed decreases
        if self.is_sliding {
            self.move_speed -= 0.2;
        }

        //ending slide (either cause not pressed or slowed down)
        if (!window.is_key_down(Key::Down) && self.is_sliding) || ((self.move_speed <= SPRINT_SPEED) && self.is_sliding){
            self.is_sliding = false;
            self.mover.foot_level +=CROUCH_Distance;
            self.last_input=No;
            self.slice_cooldown = SLIDE_COOLDOWN_TIME;
        }

        

        if window.is_key_pressed(Key::G,KeyRepeat::No) {
            self.godmode = !self.godmode;
        }

        //adjust feet_level to fit floor_level
        // smoothing: only "catch up" foot level with floor level at s smooting speed
        if !self.godmode || window.is_key_down(Key::Y) {
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
        self.velocity_x = self.mover.facing_direction.cos() * ROTATION_SPEED_MOUSE;
        self.velocity_y = self.mover.facing_direction.sin() * ROTATION_SPEED_MOUSE;
    }
}
