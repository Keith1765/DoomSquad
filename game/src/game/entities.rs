use minifb::{Key, Window};

use crate::game::entities::EntityType::*;
use crate::game::entity_behaviour::{
    archer_behaviour, dummy_behaviour, enemy_arrow_behaviour, enemy_bullet_behaviour,
    exploded_red_barrel_behaviour, melee_enemy_behaviour, player_arrow_behaviour,
    player_bullet_behaviour, ranged_enemy_behaviour, red_barrel_behaviour,
    summoner_enemy_behaviour, weak_enemy_behaviour,
};

use crate::game::map::{Map, Point};
use crate::game::movement::Mover;
use crate::render::RendererData;
use crate::render::sprites::Sprite;
use crate::render::sprites::WalkCycleHandler;

use std::fmt;

pub const ENTITY_DEFAULT_VIEW_HEIGHT: f64 = 15.0;
const ENTITY_MOVEMENT_SMOOTHING_SPEED: f64 = 1.5;
const GRAVITY_CONST: f64 = -0.8;
pub const BULLET_SPEED: f64 = 30.0;
pub const SHOOTING_COOLDOWN: i32 = 50;
pub const ARROW_COOLDOWN: i32 = 10;
pub const BULLET_COOLDOWN: i32 = 5;
pub const SUMMONING_COOLDOWN: i32 = 200;
pub const PROJECTILE_HP: f64 = 30.0;
pub const ENEMY_HP: f64 = 50.0;
pub const ENEMY_SIZE: f64 = 20.0; //testing
pub const BULLET_DMG: f64 = 20.0;
pub const DUMMY_HP: f64 = 100.0;
pub const DUMMY_SIZE: f64 = 15.0;
pub const WEAK_ENEMY_MULTIPLICATOR: f64 = 0.5;
pub const RED_BARREL_HP: f64 = 1.0;
pub const RED_BARREL_SIZE: f64 = 30.0;
pub const BULLET_TRAVEL_COUNTDOWN: i32 = 2;
pub const ARROW_SPEED: f64 = 15.0;
pub const ARROW_DMG: f64 = 40.0;
pub const EXPLODED_RED_BARREL_HP: f64 = 14.0;
pub const RED_BARREL_DMG: f64 = 1000.0;
pub const EXPLODED_RED_BARREL_SIZE: f64 = 20000.0;
pub const MELEE_ENEMY_ATTACK_COOLDOWN: i32 = 20;
pub const MEELE_ENEMY_DMG: f64 = 25.0;
pub const MELEE_ENEMY_RANGE: f64 = 5.0;

#[derive(Clone, PartialEq, Eq)]
pub enum EntityType {
    Dummy,        //sprite done
    PlayerBullet, //sprite done
    EnemyBullet,  //sprites done
    RedBarrel,    //sprite done
    ExplodedRedBarrel,
    RangedEnemy,
    MeleeEnemy, //sprite done
    SummonerEnemy,
    WeakEnemy,
    EnemyArrow,  //sprite done
    PlayerArrow, //sprite done
    Archer,
    Button,
}
impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            EntityType::Dummy => "Dummy",
            EntityType::PlayerBullet => "PlayerBullet",
            EntityType::EnemyBullet => "EnemyBullet",
            EntityType::RedBarrel => "RedBarrel",
            EntityType::ExplodedRedBarrel => "ExplodedRedBarrel",
            EntityType::RangedEnemy => "RangedEnemy",
            EntityType::MeleeEnemy => "MeleeEnemy",
            EntityType::SummonerEnemy => "SummonerEnemy",
            EntityType::WeakEnemy => "WeakEnemy",
            EntityType::EnemyArrow => "EnemyArrow",
            EntityType::PlayerArrow => "PlayerArrow",
            EntityType::Archer => "Archer",
            //TODO adding button behiaviour and also interect in player
            EntityType::Button => "Button",
        };

        write!(f, "{}", text)
    }
}
impl EntityType {
    pub fn get_walk_animation_data(&self) -> Option<(usize, usize, usize)> {
        match self {
            // TODO insert proper values here
            EntityType::MeleeEnemy => Some((9, 10, 8)),
            EntityType::WeakEnemy => Some((17, 18, 8)),
            EntityType::Archer => Some((0, 0, 8)),
            _ => None, // other types do not have walking animations
        }
    }

    pub fn get_action_animation_data(&self) -> Option<(usize, usize)> {
        match self {
            // TODO insert proper values here
            EntityType::MeleeEnemy => Some((7, 15)),
            EntityType::WeakEnemy => Some((15, 15)),
            EntityType::SummonerEnemy => Some((19, 25)),
            EntityType::RangedEnemy => Some((11, 15)),
            EntityType::Archer => Some((0, 15)),
            _ => None, // other types do not have walking animations
        }
    }

    pub fn get_default_texture_id(&self) -> usize {
        match self {
            // TODO insert proper values here
            EntityType::MeleeEnemy => 8,
            EntityType::WeakEnemy => 16,
            EntityType::SummonerEnemy => 20,
            EntityType::RangedEnemy => 12,
            EntityType::Archer => 0,
            EntityType::Dummy => 5,
            EntityType::RedBarrel => 13,
            EntityType::ExplodedRedBarrel => 6,
            EntityType::EnemyBullet => 3,
            EntityType::PlayerBullet => 4,
            EntityType::EnemyArrow => 2,
            EntityType::PlayerArrow => 1,
            EntityType::Button => 0,
            _ => 0,
        }
    }
}
#[derive(Clone)]
pub enum EntityEvent {
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
    pub action_cooldown: i32,
    pub hp: f64,
    pub size: f64,
    pub did_damage: bool,
    pub vertical_aim: f64,
}

impl Entity {
    pub fn new(
        position: Point,
        start_floor_level: f64,
        start_foot_level: f64,
        collision_height: f64,
        facing_direction: f64,
        sprite_texture_id: usize,
        renderer_data: &RendererData,
        entity_type: EntityType,
        hp: f64,
        size: f64,
        vertical_aim: f64,
    ) -> Option<Self> {
        let entity = Entity {
            mover: Mover {
                position, // Point { x: 230.0, y: 210.0 },
                floor_level: start_floor_level,
                foot_level: start_foot_level,
                view_level: ENTITY_DEFAULT_VIEW_HEIGHT,
                height: collision_height,
                facing_direction,
            },
            movement_locked: false,
            sprite: Sprite {
                default_texture_id: sprite_texture_id,
                height: renderer_data.textures.get(&sprite_texture_id)?.height as f64,
                width: renderer_data.textures.get(&sprite_texture_id)?.width as f64,
                action_sprite_switcher: None,
                walk_cycle_handler: None,
                },
            gravity: GRAVITY_CONST,
            vertical_velocity: 0.0,
            entity_type: entity_type,
            orientation_lock: false,
            action_cooldown: 0,
            hp: hp,
            size: size,
            did_damage: false,
            vertical_aim: vertical_aim,
        };
        Some(entity)
    }

    //updates every tick
    pub fn update(
        self: &mut Self,
        window: &Window,
        map: &Map,
        player_mover: &Mover,
        renderer_data: &RendererData,
    ) -> Vec<EntityEvent> {
        //collect all Spawn events for gamestate
        let mut events: Vec<EntityEvent> = Vec::new();
        if window.is_key_pressed(Key::L, minifb::KeyRepeat::No) {
            self.movement_locked = !self.movement_locked;
        }

        if self.movement_locked {
            return events;
        }

        //match entity behaviour
        match self.entity_type {
            Dummy => dummy_behaviour(self, map),
            PlayerBullet => player_bullet_behaviour(self, map),
            EnemyBullet => enemy_bullet_behaviour(self, map),
            RedBarrel => red_barrel_behaviour(self, map),
            ExplodedRedBarrel => exploded_red_barrel_behaviour(self),
            RangedEnemy => {
                ranged_enemy_behaviour(self, map, player_mover.position, renderer_data, &mut events)
            }
            Archer => {
                archer_behaviour(self, map, player_mover.position, renderer_data, &mut events)
            }
            MeleeEnemy => melee_enemy_behaviour(self, map, player_mover.position),
            SummonerEnemy => summoner_enemy_behaviour(
                self,
                map,
                player_mover.position,
                renderer_data,
                &mut events,
            ),
            WeakEnemy => weak_enemy_behaviour(self, map, player_mover.position),
            EnemyArrow => enemy_arrow_behaviour(self, map),
            PlayerArrow => player_arrow_behaviour(self, map),
            _ => dummy_behaviour(self, map),
        }

        //set view level correctly (excluded projectiles)
        if !matches!(
            self.entity_type,
            EntityType::EnemyArrow
                | EntityType::EnemyBullet
                | EntityType::PlayerArrow
                | EntityType::PlayerBullet
        ) {
            self.mover.view_level = self.mover.foot_level + ENTITY_DEFAULT_VIEW_HEIGHT;
        }


        if let Some(switcher) = &mut self.sprite.action_sprite_switcher {
            if switcher.countdown == 0 {
                self.sprite.action_sprite_switcher = None;
            } else {
                switcher.countdown -= 1;
            }
        }

        return events;
    }

    //std movement constantly moves towards player
    pub fn normal_enemy_movement(
        self: &mut Self,
        map: &Map,
        player_position: Point,
        move_speed: f64,
    ) {
        if !self.orientation_lock {
            self.mover.facing_direction = self.mover.position.angle_to(&player_position);
        }
        //entities never move when very close to player
        if self.mover.position.distance_to(&player_position) > self.size + MELEE_ENEMY_RANGE - 0.1 {
            let step_succesful = self.mover.step(move_speed, 0.0, map, false);
            if step_succesful {
                self.sprite.continue_or_start_walk_cycle(&self.entity_type);
            } else {
                self.sprite.walk_cycle_handler = None; // if cant step, stop walk animation
            }
        }
    }

    //percentage var should be 1.0 per default, lower for small gravity effect
    pub fn gravity(self: &mut Self, map: &Map, percentage: f64) {
        // GRAVITY
        //adjust for gravity
        self.vertical_velocity += self.gravity * percentage;

        //vertical movement after gravity adjustment
        self.mover.foot_level += self.vertical_velocity;

        //landing
        if self.mover.foot_level <= self.mover.floor_level {
            self.mover.foot_level = self.mover.floor_level;
            self.vertical_velocity = 0.0;
        }
        self.mover.step(0.0, 0.0, map, false);
    }
}
