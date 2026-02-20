use core::f64;

use crate::{
    game::{
        entities::{
            DUMMY_HP, DUMMY_SIZE, ENEMY_HP, ENEMY_SIZE, EXPLODED_RED_BARREL_SIZE, Entity,
            EntityType, PROJECTILE_HP, RED_BARREL_HP, RED_BARREL_SIZE, WEAK_ENEMY_MULTIPLICATOR,
        },
        map::Point,
    }, render::RendererData
};

pub fn generate_entities(
    entity_type: EntityType,
    position: Point,
    height: f64,
    facing_direction: f64,
    renderer_data: &RendererData,
) -> Entity {
    match entity_type {
        EntityType::PlayerBullet => Entity::new(
            position,
            height,
            1.0,
            facing_direction,
            1,
            renderer_data,
            EntityType::PlayerBullet,
            PROJECTILE_HP,
            1.0,
        )
        .unwrap(),
        EntityType::EnemyBullet => Entity::new(
            position,
            height,
            1.0,
            facing_direction,
            1,
            renderer_data,
            EntityType::EnemyBullet,
            PROJECTILE_HP,
            1.0,
        )
        .unwrap(),
        EntityType::PlayerArrow => Entity::new(
            position,
            height,
            1.0,
            facing_direction,
            5,
            renderer_data,
            EntityType::PlayerArrow,
            PROJECTILE_HP,
            1.0,
        )
        .unwrap(),
        EntityType::EnemyArrow => Entity::new(
            position,
            height,
            1.0,
            facing_direction,
            5,
            renderer_data,
            EntityType::EnemyArrow,
            PROJECTILE_HP,
            1.0,
        )
        .unwrap(),
        EntityType::Dummy => Entity::new(
            position,
            height,
            DUMMY_SIZE,
            facing_direction,
            0,
            renderer_data,
            EntityType::Dummy,
            DUMMY_HP,
            DUMMY_SIZE,
        )
        .unwrap(),
        EntityType::RangedEnemy => Entity::new(
            position,
            height,
            ENEMY_SIZE,
            facing_direction,
            3,
            renderer_data,
            EntityType::RangedEnemy,
            ENEMY_HP,
            ENEMY_SIZE,
        )
        .unwrap(),
        EntityType::Archer => Entity::new(
            position,
            height,
            ENEMY_SIZE,
            facing_direction,
            3,
            renderer_data,
            EntityType::Archer,
            ENEMY_HP,
            ENEMY_SIZE,
        )
        .unwrap(),
        EntityType::MeleeEnemy => Entity::new(
            position,
            height,
            ENEMY_SIZE,
            facing_direction,
            4,
            renderer_data,
            EntityType::MeleeEnemy,
            ENEMY_HP,
            ENEMY_SIZE,
        )
        .unwrap(),
        EntityType::SummonerEnemy => Entity::new(
            position,
            height,
            ENEMY_SIZE,
            facing_direction,
            6,
            renderer_data,
            EntityType::SummonerEnemy,
            ENEMY_HP,
            ENEMY_SIZE,
        )
        .unwrap(),
        EntityType::WeakEnemy => Entity::new(
            position,
            height,
            ENEMY_SIZE * WEAK_ENEMY_MULTIPLICATOR,
            facing_direction,
            7,
            renderer_data,
            EntityType::WeakEnemy,
            ENEMY_HP * WEAK_ENEMY_MULTIPLICATOR,
            ENEMY_SIZE * WEAK_ENEMY_MULTIPLICATOR,
        )
        .unwrap(),
        EntityType::RedBarrel => Entity::new(
            position,
            height,
            RED_BARREL_SIZE,
            facing_direction,
            2,
            renderer_data,
            EntityType::RedBarrel,
            RED_BARREL_HP,
            RED_BARREL_SIZE,
        )
        .unwrap(),
        EntityType::ExplodedRedBarrel => Entity::new(
            position,
            height,
            RED_BARREL_SIZE,
            facing_direction,
            12,
            renderer_data,
            EntityType::ExplodedRedBarrel,
            RED_BARREL_HP,
            EXPLODED_RED_BARREL_SIZE,
        )
        .unwrap(),
        _ => Entity::new(
            position,
            height,
            1.0,
            facing_direction,
            0,
            renderer_data,
            EntityType::Dummy,
            DUMMY_HP,
            DUMMY_SIZE,
        )
        .unwrap(),
    }
}
