use crate::game::entities::{
    ARROW_COOLDOWN, ARROW_SPEED, BULLET_SPEED, ENEMY_HP, ENEMY_SIZE, Entity, EntityEvent,
    MELEE_ENEMY_ATTACK_COOLDOWN, SHOOTING_COOLDOWN, SUMMONING_COOLDOWN,
};

use crate::game::entities::EntityEvent::*;
use crate::game::entities::EntityType::*;
use crate::game::generate_entities::generate_entities;
use crate::game::map::{Map, Point};
use crate::game::player::MOVE_SPEED;
use crate::render::RendererData;

pub fn dummy_behaviour(entity: &mut Entity, map: &Map) {
    entity.gravity(map, 1.0);
}
pub fn player_bullet_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    entity.mover.step(BULLET_SPEED, 0.0, map, false);
}
//atm the same as player bullets
pub fn enemy_bullet_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    entity.mover.step(BULLET_SPEED, 0.0, map, false);
}

pub fn player_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.gravity(map, 0.1);
    //terminate arrow when hits the floor
    if entity.mover.foot_level <= entity.mover.floor_level {
        entity.hp = 0.0;
    }
}
//atm the same as player bullets
pub fn enemy_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.gravity(map, 0.1);
    //terminate arrow when hits the floor
    if entity.mover.foot_level <= entity.mover.floor_level {
        entity.hp = 0.0;
    }
}

pub fn red_barrel_behaviour(entity: &mut Entity, map: &Map) {
    entity.gravity(map, 1.0);
}

pub fn exploded_red_barrel_behaviour(entity: &mut Entity) {
    entity.hp -= 1.0;
}

pub fn ranged_enemy_behaviour(
    entity: &mut Entity,
    map: &Map,
    player_position: Point,
    renderer_data: &RendererData,
    events: &mut Vec<EntityEvent>,
) {
    entity.gravity(map, 1.0);

    if entity.cooldown != 0 {
        entity.cooldown -= 1;
    } else {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.cooldown = SHOOTING_COOLDOWN;
        let bullet = generate_entities(
            EnemyBullet,
            entity.mover.position,
            15.0, //per default entities are generated at default view height plus this value, therfore should be 0.0
            direction_to_player,
            renderer_data,
        );
        events.push(Spawn(bullet));
    }
}

pub fn archer_behaviour(
    entity: &mut Entity,
    map: &Map,
    player_position: Point,
    renderer_data: &RendererData,
    events: &mut Vec<EntityEvent>,
) {
    entity.gravity(map, 1.0);

    if entity.cooldown != 0 {
        entity.cooldown -= 1;
    } else {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.cooldown = ARROW_COOLDOWN;
        let arrow = generate_entities(
            EnemyArrow,
            entity.mover.position,
            entity.mover.height,
            direction_to_player,
            renderer_data,
        );
        events.push(Spawn(arrow));
    }
}

pub fn summoner_enemy_behaviour(
    entity: &mut Entity,
    map: &Map,
    player_position: Point,
    renderer_data: &RendererData,
    events: &mut Vec<EntityEvent>,
) {
    entity.gravity(map, 1.0);

    if entity.cooldown == 0 {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.cooldown = SUMMONING_COOLDOWN;
        let melee_enemy = Entity::new(
            entity.mover.position,
            entity.mover.floor_level,
            0.0,
            entity.mover.height,
            direction_to_player,
            7,
            renderer_data,
            MeleeEnemy,
            ENEMY_HP,
            ENEMY_SIZE,
        );
        if let Some(melee_enemy) = melee_enemy {
            events.push(Spawn(melee_enemy));
        }
    } else {
        entity.cooldown -= 1;
    }
}

pub fn melee_enemy_behaviour(entity: &mut Entity, map: &Map, player_position: Point) {
    entity.gravity(map, 1.0);

    entity.normal_enemy_movement(map, player_position, MOVE_SPEED);

    if entity.cooldown > 0 {
        entity.cooldown -= 1;
        return;
    }

    if entity.did_damage {
        entity.cooldown = MELEE_ENEMY_ATTACK_COOLDOWN;
        entity.did_damage = false;
    }
}

pub fn weak_enemy_behaviour(entity: &mut Entity, map: &Map, player_position: Point) {
    entity.gravity(map, 1.0);
    entity.normal_enemy_movement(map, player_position, MOVE_SPEED * 0.5);

    if entity.cooldown > 0 {
        entity.cooldown -= 1;
        return;
    }

    if entity.did_damage {
        entity.cooldown = MELEE_ENEMY_ATTACK_COOLDOWN;
        entity.did_damage = false;
    }
}

pub fn death_behaviour(entity: &mut Entity, renderer_data: &RendererData) -> Vec<EntityEvent> {
    let mut events = Vec::new();
    match entity.entity_type {
        RedBarrel => {
            let explosion = generate_entities(
                ExplodedRedBarrel,
                entity.mover.position,
                entity.mover.height,
                0.0,
                renderer_data,
            );
            events.push(Spawn(explosion));
        }

        _ => {}
    };

    events
}

//TODO @Kamil
// pub fn button_behaviour (&mut self) -> Vec<EntityEvent> {
//     let mut events = Vec::new();

//     events
// }
