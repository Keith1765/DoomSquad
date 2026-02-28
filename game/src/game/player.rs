use super::map::Map;
use crate::{SCREEN_HEIGHT, audio::audio_handler::Audio, game::{entities::{
    ARROW_COOLDOWN, BULLET_COOLDOWN, EntityEvent::{self, Spawn}, EntityType::{PlayerArrow, PlayerBullet}
}, movement::find_blocks_were_currently_in}};
use crate::game::generate_entities::generate_entities;
use crate::game::player::LastInputDirection::*;

use crate::{
    SCREEN_WIDTH,
    game::{map::Point, movement::Mover},
    render::RendererData,
};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::f64::consts::PI;

const ROTATION_SPEED_MOUSE: f64 = 2.0;
const ROTATION_SPEED_KEYS: f64 = 0.15;
pub const MOVE_SPEED: f64 = 3.0;
const FLY_UP_DOWN_SPEED: f64 = 1.0;
const MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
pub const MAX_STEP_UP_HEIGHT: f64 = 6.0;
const PLAYER_HEAD_HEIGHT: f64 = 20.0;
pub const PLAYER_VIEW_HEIGHT: f64 = 15.0;
const SPRINT_SPEED: f64 = 5.0;
const CROUCH_HEIGHT_DIFF: f64 = 5.0;
const SLIDE_COOLDOWN_TIME: i32 = 10;
const ROCKETLAUNCHER_COOLDOWN_TIME: i32 = 100;
const STRAIFING_SPEED: f64 = 0.035;
const JUMP_STRENGTH: f64 = 3.0;
const GRAVITY_CONST: f64 = -0.8;
const PLAYER_HP: f64 = 100.0; 
const PLAYER_SIZE: f64 = 3.0;
const JUMP_SPEED_BOOST_MULTIPLICATOR: f64 = 0.4;
const JUMP_SPEED_BOOST: f64 = 0.0;
const INCREASED_STRAFING_SPEED_RL: f64 = 1.5;
const ROCKETLAUNCHER_SPEED_BOOST: f64 = 5.0;
const ROCKETLAUNCHER_HEIGHT_BOOST: f64 = 5.0;
const JUMPING_ALLOWED_TIMER_AMOUNT: i32 = 10;
const DISTANCE_TO_FLOOR_WHILE_ALLOWED_JUMPING: f64 = 0.3;
const PROJECTILE_OFFSET_TO_MATCH_SCREEN_MIDDLE: f64 = 3.0;
const VERTICAL_AIM_SPEED: f64 = 0.12;
const MOUSE_SENSE_X: f64 = 0.003;
const MOUSE_SENSE_Y: f64 = 0.003;
const AIM_MODE_SLOWDOWN: f64 = 0.1;
const MOUSE_ENABLED: bool = false;

#[derive(Clone, PartialEq, Eq)]
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
    pub last_mouse_y: f32,
    pub godmode: bool,
    pub move_speed: f64,
    pub is_sliding: bool,
    pub last_input: LastInputDirection,
    pub slide_cooldown: i32,
    pub is_jumping: bool,
    pub vertical_velocity: f64,
    pub gravity: f64,
    pub rocketlauncher_cooldown: i32,
    pub hp: f64,
    pub arrow_cooldown: i32,
    pub bullet_cooldown: i32,
    pub size: f64,
    pub using_rocketlauncher: bool,
    pub interacting: bool,
    pub jumping_allowed: bool,
    pub jumping_allowed_timer: i32,
    pub vertcal_aim: f64,
    pub aim_mode: bool,
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
            last_mouse_y: SCREEN_HEIGHT as f32 / 2.0,
            godmode: false, // allows flying up and down, no collision (when those are implemented)
            move_speed: 1.0,
            is_sliding: false,
            last_input: No,
            slide_cooldown: 0,
            is_jumping: false,
            vertical_velocity: 0.0,
            gravity: -1.0,
            rocketlauncher_cooldown: 0,
            hp: PLAYER_HP,
            arrow_cooldown: 0,
            bullet_cooldown: 0,
            size: PLAYER_SIZE,
            using_rocketlauncher: false,
            interacting: false,
            jumping_allowed: false,
            jumping_allowed_timer: 0,
            vertcal_aim: 0.0,
            aim_mode: false,
        }
    }

    pub fn new_with_position(position: Point) -> Self {
        let pa: f64 = 4.0;
        Self {
            mover: Mover {
                position,
                floor_level: 0.0,
                foot_level: 0.0,
                view_level: PLAYER_VIEW_HEIGHT,
                height: PLAYER_HEAD_HEIGHT,
                facing_direction: pa,
            },
            velocity_x: pa.cos() * ROTATION_SPEED_MOUSE,
            velocity_y: pa.sin() * ROTATION_SPEED_MOUSE,
            last_mouse_x: SCREEN_WIDTH as f32 / 2.0,
            last_mouse_y: SCREEN_HEIGHT as f32 / 2.0,
            godmode: false, // allows flying up and down, no collision (when those are implemented)
            move_speed: 1.0,
            is_sliding: false,
            last_input: No,
            slide_cooldown: 0,
            is_jumping: false,
            vertical_velocity: 0.0,
            gravity: -1.0,
            rocketlauncher_cooldown: 0,
            hp: PLAYER_HP,
            arrow_cooldown: 0,
            size: PLAYER_SIZE,
            using_rocketlauncher: false,
            interacting: false,
            jumping_allowed: false,
            jumping_allowed_timer: 0,
            bullet_cooldown: 0,
            vertcal_aim: 0.0,
            aim_mode: false
        }
    }

    pub fn update(
        &mut self,
        window: &Window,
        map: &Map,
        renderer_data: &RendererData,
        audio: &mut Audio,
    ) -> Vec<EntityEvent> {
        let mut events: Vec<EntityEvent> = Vec::new();
        //reseting keyinput idfk how to do it an other way
        self.interacting = false;
        if window.is_key_pressed(Key::F, KeyRepeat::No) {
            self.interacting = true;
        }

        //swap between aim_mode and not, during aim mode sense is lowered
        if window.is_key_pressed(Key::E, KeyRepeat::No) {
            match self.aim_mode {
                true => self.aim_mode = false,
                false => self.aim_mode = true,
            }
        }

        if (window.is_key_pressed(Key::RightCtrl, KeyRepeat::No) ||( window.get_mouse_down(MouseButton::Left)&& MOUSE_ENABLED)) && self.bullet_cooldown == 0 {
            audio.play_sfx("shoot", 1.0);
            let bullet = generate_entities(
                PlayerBullet,
                self.mover.position,
                self.mover.view_level - PROJECTILE_OFFSET_TO_MATCH_SCREEN_MIDDLE, 
                self.mover.facing_direction,
                renderer_data,
                self.vertcal_aim
            );
            if let Some(bullet) = bullet {
                events.push(Spawn(bullet));
                self.bullet_cooldown = BULLET_COOLDOWN;
            }
        }

         if self.bullet_cooldown > 0 {
            self.bullet_cooldown -= 1;
        }

        if (window.is_key_pressed(Key::RightShift, KeyRepeat::No) || (window.get_mouse_down(MouseButton::Right) && MOUSE_ENABLED))&& self.arrow_cooldown == 0 {
            audio.play_sfx("arrow", 2.0);
            let arrow = generate_entities(
                PlayerArrow,
                self.mover.position,
                self.mover.height - PROJECTILE_OFFSET_TO_MATCH_SCREEN_MIDDLE,
                self.mover.facing_direction,
                renderer_data,
                self.vertcal_aim,
            );
            if let Some(arrow) = arrow {events.push(Spawn(arrow));}
            self.arrow_cooldown = ARROW_COOLDOWN;
        }

        if self.arrow_cooldown > 0 {
            self.arrow_cooldown -= 1;
        }

        //f32 cause window.get_mouse_pos gives us f32
        if let Some((mx,my)) = window.get_mouse_pos(MouseMode::Pass)  && MOUSE_ENABLED {
            self.check_angle();
            let dx = mx - self.last_mouse_x; // mouse delta
            let dy = my - self.last_mouse_y; // mouse delta
            self.mover.facing_direction += dx as f64 * MOUSE_SENSE_X; // sensitivity

            self.vertcal_aim = (self.vertcal_aim + dy  as f64 * MOUSE_SENSE_Y ).clamp(-1.0, 1.0 );

            self.last_mouse_x = mx; // store for next frame
            self.last_mouse_y = my;
            self.update_dir();
        }

        if window.is_key_down(Key::Left) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding || self.is_jumping {
                true => match self.using_rocketlauncher {
                    true => STRAIFING_SPEED * INCREASED_STRAFING_SPEED_RL,
                    false => STRAIFING_SPEED,
                },
                false => match self.aim_mode {
                    false => ROTATION_SPEED_KEYS,
                    true => ROTATION_SPEED_KEYS * AIM_MODE_SLOWDOWN,
            },
        };

            self.check_angle();
            self.mover.facing_direction -= rotation_factor;
            self.update_dir();
        }

        if window.is_key_down(Key::Right) {
            //during slide heavily restricted rotation
            let rotation_factor = match self.is_sliding || self.is_jumping {
                true => match self.using_rocketlauncher {
                    true => STRAIFING_SPEED * INCREASED_STRAFING_SPEED_RL,
                    false => STRAIFING_SPEED,
                },
                false =>match self.aim_mode {
                    false => ROTATION_SPEED_KEYS,
                    true => ROTATION_SPEED_KEYS * AIM_MODE_SLOWDOWN,
                },
            };
            self.check_angle();
            self.mover.facing_direction += rotation_factor;
            self.update_dir();
        }

        let mut step_successful: bool = false;

        if (window.is_key_down(Key::W)
            && ((!self.is_sliding && !self.is_jumping) || self.last_input == W))
            || (self.is_jumping && (self.last_input == W))
        {
            step_successful = self.mover.step(self.move_speed, 0.0, map, self.godmode);
        }
        if (window.is_key_down(Key::A)
            && ((!self.is_sliding && !self.is_jumping) || self.last_input == A))
            || (self.is_jumping && (self.last_input == A))
        {
            step_successful = self.mover
                .step(self.move_speed, -PI / 2.0, map, self.godmode);
        }
        if (window.is_key_down(Key::D)
            && ((!self.is_sliding && !self.is_jumping) || self.last_input == D))
            || (self.is_jumping && (self.last_input == D))
        {
            step_successful = self.mover
                .step(self.move_speed, PI / 2.0, map, self.godmode);
        }

        if (window.is_key_down(Key::S)
            && ((!self.is_sliding && !self.is_jumping) || self.last_input == S))
            || (self.is_jumping && (self.last_input == S))
        {
            step_successful = self.mover.step(self.move_speed, PI, map, self.godmode);
        }

        // if we moved, play step 
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if step_successful && !self.is_sliding && !((self.mover.foot_level - self.mover.floor_level).abs() > 0.3 ) {
            audio.play_step(1.0);
        }

        if window.is_key_down(Key::Space) && self.godmode {
            self.mover.foot_level += FLY_UP_DOWN_SPEED;
        }

        if window.is_key_down(Key::LeftShift) && self.godmode {
            self.mover.foot_level -= FLY_UP_DOWN_SPEED;
        }

        //slowdown movespeed if not sprinting and sliding anymore
        if !window.is_key_down(Key::LeftShift) && !self.is_jumping && !self.is_sliding {
            self.move_speed = MOVE_SPEED;
        }

        //implementing Sprint that gradually increases movement speed
        if window.is_key_down(Key::LeftShift) && !self.godmode {
            if self.move_speed < SPRINT_SPEED - 0.1 {
                self.move_speed += 0.1;
            }
            if self.move_speed < SPRINT_SPEED {
                self.move_speed = SPRINT_SPEED;
            }

            if self.move_speed > SPRINT_SPEED && !self.is_sliding && !self.is_jumping {
                self.move_speed = SPRINT_SPEED;
            }
        }

        if (self.slide_cooldown > 0) && !window.is_key_down(Key::C) {
            self.slide_cooldown -= 1;
        }
        //init slide gives speed boost
        if window.is_key_down(Key::C)
            && !self.is_sliding
            && !self.godmode
            && self.slide_cooldown == 0
            && self.move_speed > 1.5 
        {
                audio.play_sfx("slide", 1.0);
                self.move_speed += 5.0;
                self.is_sliding = true;
                self.save_input(window);
        }

        //during slide speed decreases
        if self.is_sliding {
            self.move_speed -= 0.2;
        }

        //ending slide (either cause not pressed or slowed down)
        #[allow(clippy::nonminimal_bool)]
        if (!window.is_key_down(Key::C) && self.is_sliding)
            || ((self.move_speed <= SPRINT_SPEED) && self.is_sliding)
        {
            self.is_sliding = false;
            self.last_input = No;
            self.slide_cooldown = SLIDE_COOLDOWN_TIME;
        }

        //timer to allow jump slightly after leaving allowed window
        if (self.mover.foot_level - self.mover.floor_level).abs() < 0.3 {
            self.jumping_allowed = true;
            self.jumping_allowed_timer = JUMPING_ALLOWED_TIMER_AMOUNT;
        }

        if self.jumping_allowed_timer > 0 {
            self.jumping_allowed_timer -= 1;
        } else {
            self.jumping_allowed = false;
        }

        if self.is_jumping {
            self.jumping_allowed_timer = 0
        };

        //jumping init
        if (window.is_key_pressed(Key::Space, KeyRepeat::No)
            && ((!self.is_jumping
                && (self.mover.foot_level - self.mover.floor_level).abs()
                    < DISTANCE_TO_FLOOR_WHILE_ALLOWED_JUMPING)
                || self.jumping_allowed))
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

                audio.play_sfx("jump", 1.0);
                
                self.move_speed += self.move_speed * JUMP_SPEED_BOOST_MULTIPLICATOR;
                let speed_bonus = self.move_speed * 0.8;
                self.vertical_velocity = JUMP_STRENGTH + speed_bonus;
                self.gravity += self.gravity * (self.move_speed * 0.08) //special Relativity (kidding, just wanted to decrease height scalling on big jumps)
            }

            //rocketlauncher
            if window.is_key_pressed(Key::R, KeyRepeat::No) && (self.rocketlauncher_cooldown == 0) {

                audio.play_sfx("rocketlauncher", 1.0); 

                self.using_rocketlauncher = true;
                self.move_speed +=
                    self.move_speed * JUMP_SPEED_BOOST_MULTIPLICATOR + ROCKETLAUNCHER_SPEED_BOOST;
                let speed_bonus = self.move_speed * 0.8;
                self.vertical_velocity = JUMP_STRENGTH + speed_bonus + ROCKETLAUNCHER_HEIGHT_BOOST;
                self.gravity += self.gravity * (self.move_speed * 0.04);
                self.rocketlauncher_cooldown = ROCKETLAUNCHER_COOLDOWN_TIME;
            }

            self.save_input(window);
        }

        //Rocketlauncher cooldwon
        if self.rocketlauncher_cooldown != 0 {
            self.rocketlauncher_cooldown -= 1;
        }

        //height during jump
        if self.is_jumping {
            //adjust for gravity
            self.vertical_velocity += self.gravity;

            //vertical movement after gravity adjustment
            //TODO check if block is above
            self.mover.foot_level += self.vertical_velocity;

            //landing
            if self.mover.foot_level <= self.mover.floor_level {
                self.mover.foot_level = self.mover.floor_level;
                self.vertical_velocity = 0.0;
                self.is_jumping = false;
                if self.using_rocketlauncher {
                    self.using_rocketlauncher = false
                };
            }
        }

        if window.is_key_pressed(Key::G, KeyRepeat::No) {
            self.godmode = !self.godmode;
        }

        //adjust feet_level to fit floor_level
        // GRAVITY
        if !self.godmode && !self.is_jumping {
            //adjust for gravity
            self.vertical_velocity += self.gravity;

            //vertical movement after gravity adjustment
            // for that, we need to find if we'd bump our head into a block when moving up
            let blocks_were_stepping_inside =
                find_blocks_were_currently_in(self.mover.position, map);
            let mut lowest_ceiling_level = f64::MAX;
            let head_level = self.mover.foot_level + self.mover.height;
            for block in &blocks_were_stepping_inside {
                if block.bottom < lowest_ceiling_level// lower thn lowest previously found ceiling level
                && block.bottom > head_level-self.vertical_velocity
                // not totally below out head anyway
                {
                    lowest_ceiling_level = block.bottom;
                }
            }

            if (self.mover.foot_level + self.vertical_velocity)
                <= (lowest_ceiling_level - self.mover.height)
            {
                // if we didnt bump our head, we just go up normally
                self.mover.foot_level += self.vertical_velocity;
            } else {
                // if we bumped our head, we only go up to the ceiling and lose vertical velocity
                self.mover.foot_level = lowest_ceiling_level - self.mover.height;
                self.vertical_velocity = 0.0;
            }
            //landing
            if self.mover.foot_level <= self.mover.floor_level {
                self.mover.foot_level = self.mover.floor_level;
                self.vertical_velocity = 0.0;
                self.is_jumping = false;
            }
        }

        if self.is_sliding {
            self.mover.view_level = self.mover.foot_level + PLAYER_VIEW_HEIGHT - CROUCH_HEIGHT_DIFF;
        } else {
            self.mover.view_level = self.mover.foot_level + PLAYER_VIEW_HEIGHT;
        }

        //aim (inverted controls because pixel grid is inverted too)
        if window.is_key_down(Key::Up){
            match self.aim_mode {
                false => self.vertcal_aim = (self.vertcal_aim - VERTICAL_AIM_SPEED ).clamp(-1.0, 1.0 ),
                true => self.vertcal_aim = (self.vertcal_aim - VERTICAL_AIM_SPEED * AIM_MODE_SLOWDOWN ).clamp(-1.0, 1.0 ),
            }
        }

        if window.is_key_down(Key::Down){
            match self.aim_mode {
                false => self.vertcal_aim = (self.vertcal_aim + VERTICAL_AIM_SPEED ).clamp(-1.0, 1.0 ),
                true => self.vertcal_aim = (self.vertcal_aim + VERTICAL_AIM_SPEED * AIM_MODE_SLOWDOWN ).clamp(-1.0, 1.0 ),
            }
        }

        events
    }

    fn save_input(&mut self, window: &Window) {
        if window.is_key_down(Key::D) {
            self.last_input = D;
        } else if window.is_key_down(Key::A) {
            self.last_input = A;
        } else if window.is_key_down(Key::S) {
            self.last_input = S;
        } else if window.is_key_down(Key::W) {
            self.last_input = W;
        } else {
            self.last_input = No
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
        self.velocity_x = self.mover.facing_direction.cos() * ROTATION_SPEED_MOUSE;
        self.velocity_y = self.mover.facing_direction.sin() * ROTATION_SPEED_MOUSE;
    }
}
