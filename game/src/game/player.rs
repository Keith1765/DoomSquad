use super::map::Map;
use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    game::{
        map::{LEVEL_HEIGHT, Point},
        movement::Mover, player,
    }, render::RendererData,
};
use minifb::{Key, KeyRepeat, MouseMode, Window};
use std::f64::consts::PI;
use crate::game::player::LastInputDirection::*;
use crate::game::entities::{
    Entity,
    EntityEvent,
    EntityEvent::Spawn,
    EntityType::{PlayerBullet, PlayerArrow}, PROJECTILE_HP, ARROW_COOLDOWN,
};
use crate::game::generate_entities::generate_entities;

const ROTATION_SPEED_MOUSE: f64 = 2.0;
const ROTATION_SPEED_KEYS: f64 = 0.15;
pub const MOVE_SPEED: f64 = 1.5;
const FLY_UP_DOWN_SPEED: f64 = 1.0;
const MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
pub const MAX_STEP_UP_HEIGHT: f64 = 5.0;
const PLAYER_HEAD_HEIGHT: f64 = 15.0;
pub const PLAYER_VIEW_HEIGHT: f64 = 15.0;
const SPRINT_SPEED: f64 = 4.0;
const CROUCH_HEIGHT_DIFF: f64 = 5.0; 
const SLIDE_COOLDOWN_TIME: i32 = 10;
const ROCKETLAUNCHER_COOLDOWN_TIME: i32= 300;
const STRAIFING_SPEED: f64 = 0.025;
const JUMP_STRENGTH: f64 = 3.0;
const GRAVITY_CONST: f64 = -0.8;
const PLAYER_HP: f64 = 100.0;


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
    pub slide_cooldown: i32,
    pub is_jumping: bool,
    pub vertical_velocity: f64,
    pub gravity: f64,
    pub rocketlauncher_Cooldown: i32,
    pub hp: f64,
    pub arrow_cooldown: i32,
    
}

impl Player {
    pub fn new() -> Self {
        let pa: f64 = 4.0;
        Self {
            mover: Mover {
                position: Point { x: 1.0, y: 1.0 },
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
            slide_cooldown: 0,
            is_jumping: false,
            vertical_velocity: 0.0,
            gravity: -1.0,
            rocketlauncher_Cooldown: 0,
            hp: PLAYER_HP,
            arrow_cooldown: 0,
        }
    }

    pub fn update(&mut self, window: &Window, map: &Map,renderer_data: &RendererData) -> Vec<EntityEvent> {
        let mut events: Vec<EntityEvent> = Vec::new();

        if window.is_key_pressed(Key::RightCtrl, KeyRepeat::No) {
            let bullet = generate_entities(PlayerBullet, self.mover.position, self.mover.height, self.mover.facing_direction, renderer_data);
            events.push(Spawn(bullet));
        }

        if window.is_key_pressed(Key::RightShift, KeyRepeat::No) && self.arrow_cooldown == 0 {
            let arrow = generate_entities(PlayerArrow, self.mover.position, self.mover.height, self.mover.facing_direction, renderer_data);
            events.push(Spawn(arrow));
            self.arrow_cooldown = ARROW_COOLDOWN;
        }

        if self.arrow_cooldown > 0 {
            self.arrow_cooldown -= 1;
        }

        if let Some((mx, _my)) = window.get_mouse_pos(MouseMode::Pass) {
            self.check_angle();
            let dx = mx - self.last_mouse_x; // mouse delta
            self.mover.facing_direction += dx as f64 * 0.003; // sensitivity

            self.last_mouse_x = mx; // store for next frame
            self.update_dir();
        }

        if window.is_key_down(Key::Left) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding || self.is_jumping{
                true => STRAIFING_SPEED,
                false => ROTATION_SPEED_KEYS,
            };

            self.check_angle();
            self.mover.facing_direction -= rotation_factor;
            self.update_dir();
        }

        if window.is_key_down(Key::Right) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding || self.is_jumping {
                true => STRAIFING_SPEED,
                false => ROTATION_SPEED_KEYS,
            };
            self.check_angle();
            self.mover.facing_direction += rotation_factor;
            self.update_dir();
        }

        if (window.is_key_down(Key::W) && ((!self.is_sliding && !self.is_jumping) || self.last_input==W)) || (self.is_jumping && (self.last_input == W)) {
            self.mover.step(self.move_speed, 0.0, map, self.godmode);
            
        }
        if (window.is_key_down(Key::A) && ((!self.is_sliding && !self.is_jumping)|| self.last_input==A)) || (self.is_jumping && (self.last_input == A)) {
            self.mover.step(self.move_speed, -PI / 2.0, map, self.godmode);
        }
        if (window.is_key_down(Key::D) && ((!self.is_sliding && !self.is_jumping) || self.last_input==D)) || (self.is_jumping && (self.last_input == D)) {
            self.mover.step(self.move_speed, PI / 2.0, map, self.godmode);
        }

        if (window.is_key_down(Key::S) && ((!self.is_sliding && !self.is_jumping) || self.last_input==S)) || (self.is_jumping && (self.last_input == S)) {
            self.mover.step(self.move_speed, PI, map, self.godmode);
        }


        if window.is_key_down(Key::Space) && self.godmode {
            self.mover.foot_level += FLY_UP_DOWN_SPEED;
        }

        if window.is_key_down(Key::LeftShift) && self.godmode {
            self.mover.foot_level -= FLY_UP_DOWN_SPEED;
        }

        //slowdown movespeed if not sprinting and sliding anymore
        if !window.is_key_down(Key::LeftShift) && !window.is_key_down(Key::Down) && !self.is_jumping{
            self.move_speed=MOVE_SPEED;
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

        if (self.slide_cooldown > 0) && !window.is_key_down(Key::C) {
            self.slide_cooldown -= 1;
        } 
        //init slide gives speed boost
        if window.is_key_down(Key::C) && !self.is_sliding && !self.godmode && self.slide_cooldown == 0{
            if self.move_speed > 1.5 {
                self.move_speed += 5.0;
                self.is_sliding = true;
                self.save_input(window);
            }
            
        }
        
        //during slide speed decreases
        if self.is_sliding {
            self.move_speed -= 0.2;
        }

        //ending slide (either cause not pressed or slowed down)
        if (!window.is_key_down(Key::C) && self.is_sliding) || ((self.move_speed <= SPRINT_SPEED) && self.is_sliding){
            self.is_sliding = false;
            self.last_input=No;
            self.slide_cooldown = SLIDE_COOLDOWN_TIME;
        }

        //jumping init
        if (window.is_key_pressed(Key::Space, KeyRepeat::No)
        && !self.is_jumping && (self.mover.foot_level-self.mover.floor_level).abs() < 0.01 )
        || window.is_key_pressed(Key::R, KeyRepeat::No)
            {
                self.gravity = GRAVITY_CONST;
                self.is_jumping = true;
                if self.is_sliding {
                    self.is_sliding = false;
                    self.mover.foot_level += CROUCH_HEIGHT_DIFF;
                    self.slide_cooldown = SLIDE_COOLDOWN_TIME;
                    self.move_speed += 3.0;
                }
                //normal jump init
                if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                    self.move_speed += self.move_speed*0.75;
                    let speed_bonus = self.move_speed * 0.8;
                    self.vertical_velocity = JUMP_STRENGTH + speed_bonus;
                    self.gravity += self.gravity*(self.move_speed*0.08) //special Relativity (kidding, just wanted to decrease height scalling on big jumps)
                }

                //rocketlauncher
                if window.is_key_pressed(Key::R, KeyRepeat::No) && (self.rocketlauncher_Cooldown == 0){
                    self.move_speed += self.move_speed*0.75 + 10.0;
                    let speed_bonus = self.move_speed * 0.8;
                    self.vertical_velocity = JUMP_STRENGTH + speed_bonus + 3.0;
                    self.gravity += self.gravity*(self.move_speed*0.04);
                    self.rocketlauncher_Cooldown = ROCKETLAUNCHER_COOLDOWN_TIME;
                }

                self.save_input(window);

                

        }

        //Rocketlauncher cooldwon
        if self.rocketlauncher_Cooldown != 0{
            self.rocketlauncher_Cooldown -= 1;
        }

        //height during jump
        if self.is_jumping {
            //adjust for gravity
            self.vertical_velocity += self.gravity;

            //vertical movement after gravity adjustment
            //TODO check if block is above
            self.mover.foot_level += self.vertical_velocity;

            //landing
            if self.mover.foot_level <= self.mover.floor_level{
                self.mover.foot_level = self.mover.floor_level;
                self.vertical_velocity= 0.0;
                self.is_jumping = false;
            }

        }

        if window.is_key_pressed(Key::G,KeyRepeat::No) {
            self.godmode = !self.godmode;
        }

        //adjust feet_level to fit floor_level
        // GRAVITY
        if !self.godmode && !self.is_jumping {
            
            //adjust for gravity
            self.vertical_velocity += self.gravity;

            //vertical movement after gravity adjustment
            self.mover.foot_level += self.vertical_velocity;

            //landing
            if self.mover.foot_level <= self.mover.floor_level{
                self.mover.foot_level = self.mover.floor_level;
                self.vertical_velocity= 0.0;
                self.is_jumping = false;
            }

        
        }
        if self.is_sliding {
            self.mover.view_level = self.mover.foot_level + PLAYER_VIEW_HEIGHT - CROUCH_HEIGHT_DIFF;
        }
        else{
            self.mover.view_level = self.mover.foot_level + PLAYER_VIEW_HEIGHT;
        }

        return events;
    }

    fn save_input (&mut self, window: &Window){
        if window.is_key_down(Key::D) {self.last_input = D};
        if window.is_key_down(Key::A) {self.last_input = A};
        if window.is_key_down(Key::S) {self.last_input = S};
        if window.is_key_down(Key::W) {self.last_input = W};
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
