use std::rc::Rc;

use minifb::{Key, Window};

use crate::game::map::{Map, Point};
use crate::game::movement::Mover;
use crate::game::player::MOVE_SPEED;
use crate::game::{Game, map};
use crate::render::RendererData;
use crate::render::sprites::Sprite;
use crate::game::entities::EntityType::*;


const ENTITY_DEFAULT_VIEW_HEIGHT: f64 = 15.0;
const ENTITY_MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
const GRAVITY_CONST: f64 = -0.8;
const BULLET_SPEED: f64 =  20.0;

#[derive(Clone, PartialEq, Eq)]
pub enum EntityType{
    Dummy,
    Bullet,
    RedBarrel,
    RangedEnemy,
    MeleeEnemy,
}

#[derive(Clone)]
pub struct Entity {
    pub mover: Mover,
    pub movement_locked: bool,
    pub sprite: Sprite,
    pub gravity: f64,
    pub vertical_velocity: f64,
    pub entity_type: EntityType,
}

impl Entity {
    pub fn new(
        position: Point,
        start_floor_level: f64,
        collision_height: f64,
        facing_direction: f64,
        sprite_texture_id: usize,
        renderer_data: &RendererData,
        type_id: i32,
    ) -> Option<Self> {
        let entity = Entity {
            mover: Mover {
                position, // Point { x: 230.0, y: 210.0 },
                floor_level: start_floor_level,
                foot_level: start_floor_level,
                view_level: start_floor_level + ENTITY_DEFAULT_VIEW_HEIGHT,
                height: collision_height,
                facing_direction,
            },
            movement_locked: true, // TODO change this; for now locked by default
            sprite: Sprite {
                texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
            },
            gravity: GRAVITY_CONST,
            vertical_velocity: 0.0,
            entity_type: entity_type_from_id(type_id),
        };
        Some(entity)
    }

    fn normal_enemy_movement(self: &mut Self, map: &Map, player_position: Point) {
        self.mover.facing_direction = self.mover.position.angle_to(&player_position);
        self.mover.step(MOVE_SPEED, 0.0, map, false);
    }





    pub fn update(self: &mut Self, window: &Window, map: &Map, player_mover: &Mover) {
        if window.is_key_pressed(Key::L,minifb::KeyRepeat::No) {
            self.movement_locked = !self.movement_locked;
        }

        if self.movement_locked {return;}

        match self.entity_type {
            Dummy   => self.dummy_behaviour(map, player_mover.position),
            Bullet   => self.bullet_behaviour(map,player_mover.position),
            RedBarrel   => self.red_barrel_behaviour(map, player_mover.position),
            RangedEnemy   => self.ranged_enemy_behaviour(map, player_mover.position),
            MeleeEnemy   => self.melee_enemy_behaviour(map, player_mover.position),
            _       => self.dummy_behaviour(map, player_mover.position),
        }

       

        //set view level correctly
        self.mover.view_level = self.mover.foot_level + ENTITY_DEFAULT_VIEW_HEIGHT;
        // // move testwise
        // self.normal_enemy_movement(map, player_mover.position);
    }


    fn dummy_behaviour (self: & mut Self, map: &Map, player_position: Point) {
        self.gravity(map);
    }
    fn bullet_behaviour (self: & mut Self, map: &Map, player_position: Point) {
        
    }
    fn red_barrel_behaviour (self: & mut Self, map: &Map, player_position: Point) {
        self.gravity(map);
    }
    fn ranged_enemy_behaviour (self: & mut Self, map: &Map, player_position: Point) {
        self.gravity(map);
    }
    fn melee_enemy_behaviour (self: & mut Self, map: &Map, player_position: Point) {
        self.gravity(map);
        self.normal_enemy_movement(map, player_position);
    }

    fn gravity (self: & mut Self, map: &Map){
        // GRAVITY
            //adjust for gravity
            self.vertical_velocity += self.gravity;
    
            //vertical movement after gravity adjustment
            self.mover.foot_level += self.vertical_velocity;
    
            //landing
            if self.mover.foot_level <= self.mover.floor_level{
                self.mover.foot_level = self.mover.floor_level;
                self.vertical_velocity= 0.0;
            }
        self.mover.step(0.0, 0.0, map, false);
    }


}

 fn entity_type_from_id(id: i32) -> EntityType {
        match id {
            1 => EntityType::Bullet,
            2 => EntityType::RedBarrel,
            3 => EntityType::MeleeEnemy,
            4 => EntityType::RangedEnemy,
            _ => EntityType::Dummy,

        }
    }
