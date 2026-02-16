use std::rc::Rc;

use minifb::{Key, Window};
use quick_xml::events;

use crate::game::map::{Map, Point};
use crate::game::movement::Mover;
use crate::game::player::MOVE_SPEED;
use crate::game::{Game, map};
use crate::render::RendererData;
use crate::render::sprites::Sprite;
use crate::game::entities::EntityType::*;
use crate::game::entities::EntityEvent::*;
use crate::game::generate_entities::generate_entities;

use std::fmt;

const ENTITY_DEFAULT_VIEW_HEIGHT: f64 = 15.0;
const ENTITY_MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
const GRAVITY_CONST: f64 = -0.8;
pub const BULLET_SPEED: f64 =  20.0;
const SHOOTING_COOLDOWN: i32 = 50;
const SUMMONING_COOLDOWN: i32 = 500;
pub const BULLET_HP: f64 = 30.0;
pub const ENEMY_HP: f64 = 50.0;
pub const ENEMY_SIZE: f64 = 10.0;
pub const BULLET_DMG: f64 = 20.0;
pub const DUMMY_HP: f64 = 100.0;
pub const DUMMY_SIZE: f64 = 15.0;
pub const WEAK_ENEMY_MULTIPLICATOR: f64 = 0.5;
pub const RED_BARREL_HP: f64 = 1.0;
pub const RED_BARREL_SIZE: f64 = 5.0;
pub const BULLET_TRAVEL_COUNTDOWN: i32 = 2;

#[derive(Clone, PartialEq, Eq)]
pub enum EntityType{
    Dummy, //sprite done
    PlayerBullet,//sprite done
    EnemyBullet,//sprites done
    RedBarrel, //sprite done
    RangedEnemy,
    MeleeEnemy,//sprite done
    SummonerEnemy,
    WeakEnemy,
}
impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            EntityType::Dummy => "Dummy",
            EntityType::PlayerBullet => "PlayerBullet",
            EntityType::EnemyBullet => "EnemyBullet",
            EntityType::RedBarrel => "RedBarrel",
            EntityType::RangedEnemy => "RangedEnemy",
            EntityType::MeleeEnemy => "MeleeEnemy",
            EntityType::SummonerEnemy => "SummonerEnemy",
            EntityType::WeakEnemy => "WeakEnemy",
        };

        write!(f, "{}", text)
    }
}
#[derive(Clone)]
pub enum EntityEvent{
    Spawn(Entity),
    
}

#[derive(Clone)]
pub struct Entity {
    pub mover: Mover,
    pub movement_locked: bool,
    pub sprite: Sprite,
    pub gravity: f64,
    pub vertical_velocity: f64,
    pub entity_type: EntityType,
    pub orientation_lock: bool,
    pub cooldown: i32,
    pub hp: f64,
    pub size: f64,
}

impl Entity {
    pub fn new(
        position: Point,
        start_floor_level: f64,
        collision_height: f64,
        facing_direction: f64,
        sprite_texture_id: usize,
        renderer_data: &RendererData,
        entity_type: EntityType,
        hp: f64,
        size: f64,
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
            movement_locked: false, 
            sprite: Sprite {
                texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
            },
            gravity: GRAVITY_CONST,
            vertical_velocity: 0.0,
            entity_type: entity_type,
            orientation_lock: false,
            cooldown: 0,
            hp: hp,
            size: size,
        };
        Some(entity)
    }

    pub fn update(self: &mut Self, window: &Window, map: &Map, player_mover: &Mover, renderer_data: &RendererData) -> Vec<EntityEvent> {
        let mut events: Vec<EntityEvent> = Vec::new();
        if window.is_key_pressed(Key::L,minifb::KeyRepeat::No) {
            self.movement_locked = !self.movement_locked;
        }

        if self.movement_locked {return events;}

        match self.entity_type {
            Dummy   => self.dummy_behaviour(map, player_mover.position, &mut events),
            PlayerBullet   => self.player_bullet_behaviour(map,player_mover.position, &mut events),
            EnemyBullet   => self.enemy_bullet_behaviour(map,player_mover.position, &mut events),
            RedBarrel   => self.red_barrel_behaviour(map, player_mover.position, &mut events),
            RangedEnemy   => self.ranged_enemy_behaviour(map, player_mover.position, renderer_data, &mut events),
            MeleeEnemy   => self.melee_enemy_behaviour(window, map, player_mover.position, &mut events),
            SummonerEnemy   => self.summoner_enemy_behaviour(map, player_mover.position, renderer_data, &mut events),
            WeakEnemy   => self.weak_enemy_behaviour(map, player_mover.position, &mut events),
            _       => self.dummy_behaviour(map, player_mover.position, &mut events),
        }

       

        //set view level correctly
        self.mover.view_level = self.mover.foot_level + ENTITY_DEFAULT_VIEW_HEIGHT;
        // // move testwise
        // self.normal_enemy_movement(map, player_mover.position);

        return events;
    }

    fn normal_enemy_movement(self: &mut Self, map: &Map, player_position: Point, move_speed: f64) {
        if !self.orientation_lock{
            self.mover.facing_direction = self.mover.position.angle_to(&player_position);
        }
        self.mover.step(move_speed, 0.0, map, false);
    }

    fn dummy_behaviour (self: & mut Self, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.gravity(map);
    }
    fn player_bullet_behaviour (self: & mut Self, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.hp -= 0.25;
        self.mover.step(BULLET_SPEED, 0.0, map, false);
    }
    //atm the same as player bullets
    fn enemy_bullet_behaviour (self: & mut Self, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.hp -= 0.25;
        self.mover.step(BULLET_SPEED, 0.0, map, false);
    }
    fn red_barrel_behaviour (self: & mut Self, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.gravity(map);
    }

    fn ranged_enemy_behaviour (self: & mut Self, map: &Map, player_position: Point, renderer_data: &RendererData, events: &mut Vec<EntityEvent>) {
        self.gravity(map);

        if self.cooldown != 0 {
            self.cooldown -= 1;
        }

        else{
            let direction_to_player = self.mover.position.angle_to(&player_position);
            self.cooldown = SHOOTING_COOLDOWN;
            let bullet = generate_entities(EnemyBullet, self.mover.position, self.mover.height, direction_to_player, renderer_data);
            events.push(Spawn(bullet));
        }
    }

    fn summoner_enemy_behaviour (self: & mut Self, map: &Map, player_position: Point, renderer_data: &RendererData, events: &mut Vec<EntityEvent>) {
        self.gravity(map);

         if self.cooldown == 20 {
            let direction_to_player = self.mover.position.angle_to(&player_position);
            let melee_enemy = Entity::new(self.mover.position, self.mover.floor_level, self.mover.height, direction_to_player, 7, renderer_data, MeleeEnemy, ENEMY_HP, ENEMY_SIZE).unwrap();
            events.push(Spawn(melee_enemy));
        }

        if self.cooldown == 10 {
            let direction_to_player = self.mover.position.angle_to(&player_position);
            let melee_enemy = Entity::new(self.mover.position, self.mover.floor_level, self.mover.height, direction_to_player, 7, renderer_data, MeleeEnemy, ENEMY_HP, ENEMY_SIZE).unwrap();
            events.push(Spawn(melee_enemy));
        }

        if self.cooldown == 0 {
            let direction_to_player = self.mover.position.angle_to(&player_position);
            self.cooldown = SUMMONING_COOLDOWN;
            let melee_enemy = Entity::new(self.mover.position, self.mover.floor_level, self.mover.height, direction_to_player, 7, renderer_data, MeleeEnemy, ENEMY_HP, ENEMY_SIZE).unwrap();
            events.push(Spawn(melee_enemy));
        }

        else{
            self.cooldown -= 1;
        }
    }

    fn melee_enemy_behaviour (self: & mut Self, window: &Window, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.gravity(map);
        if window.is_key_down(Key::B){
            self.orientation_lock= true;
        }
        else{
            self.orientation_lock = false;
        }
        self.normal_enemy_movement(map, player_position, MOVE_SPEED);
    }

    fn weak_enemy_behaviour (self: & mut Self, map: &Map, player_position: Point, events: &mut Vec<EntityEvent>) {
        self.gravity(map);
        self.normal_enemy_movement(map, player_position, MOVE_SPEED*0.5);
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

    //TODO creat spawning functions
    

}

