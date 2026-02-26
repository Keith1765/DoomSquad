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

const BULLET_FLIGHT_COEFICIENT: f64 = 7.0;
const ARROW_FLIGHT_COEFICIENT: f64 = 3.0;

pub fn dummy_behaviour(entity: &mut Entity, map: &Map) {
    entity.gravity(map, 1.0);
}
pub fn player_bullet_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(BULLET_SPEED, 0.0, map, false);
    entity.mover.foot_level -= entity.vertical_aim*BULLET_FLIGHT_COEFICIENT;

    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}

//atm the same as player bullets
pub fn enemy_bullet_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(BULLET_SPEED, 0.0, map, false);

    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}

pub fn player_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.mover.foot_level -= entity.vertical_aim*ARROW_FLIGHT_COEFICIENT;
    entity.gravity(map, 0.1);
    //terminate arrow when hits the floor
    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}
//atm the same as player bullets
pub fn enemy_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25;
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.gravity(map, 0.1);
    //terminate arrow when hits the floor
    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
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

    if entity.action_cooldown != 0 {
        entity.action_cooldown -= 1;
    } else {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.action_cooldown = SHOOTING_COOLDOWN;
        let bullet = generate_entities(
            EnemyBullet,
            entity.mover.position,
            15.0, //per default entities are generated at default view height plus this value, therfore should be 0.0
            direction_to_player,
            renderer_data,
            0.0,
        );
        events.push(Spawn(bullet));
        // attack animation
        if let Some((action_texture_id, action_cooldown)) = entity.entity_type.get_action_animation_data() {
            entity.sprite.switch_sprite_for_action(action_texture_id, action_cooldown);
        }
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

    if entity.action_cooldown != 0 {
        entity.action_cooldown -= 1;
    } else {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.action_cooldown = ARROW_COOLDOWN;
        let arrow = generate_entities(
            EnemyArrow,
            entity.mover.position,
            entity.mover.height,
            direction_to_player,
            renderer_data,
            0.0,
        );
        events.push(Spawn(arrow));
        // attack animation
        if let Some((action_texture_id, action_cooldown)) = entity.entity_type.get_action_animation_data() {
            entity.sprite.switch_sprite_for_action(action_texture_id, action_cooldown);
        }
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

    if entity.action_cooldown == 0 {
        let direction_to_player = entity.mover.position.angle_to(&player_position);
        entity.action_cooldown = SUMMONING_COOLDOWN;
        let melee_enemy = generate_entities(
            WeakEnemy,
            entity.mover.position,
            entity.mover.height,
            direction_to_player,
            renderer_data,
            0.0,
        );
        events.push(Spawn(melee_enemy));
        // attack animation
        if let Some((action_texture_id, action_cooldown)) = entity.entity_type.get_action_animation_data() {
            entity.sprite.switch_sprite_for_action(action_texture_id, action_cooldown);
        }
    } else {
        entity.action_cooldown -= 1;
    }
}

pub fn melee_enemy_behaviour(entity: &mut Entity, map: &Map, player_position: Point) {
    entity.gravity(map, 1.0);

    entity.normal_enemy_movement(map, player_position, MOVE_SPEED);

    if entity.action_cooldown > 0 {
        entity.action_cooldown -= 1;
        return;
    }

    if entity.did_damage {
        entity.action_cooldown = MELEE_ENEMY_ATTACK_COOLDOWN;
        entity.did_damage = false;
        // attack animation
        if let Some((action_texture_id, action_cooldown)) = entity.entity_type.get_action_animation_data() {
            entity.sprite.switch_sprite_for_action(action_texture_id, action_cooldown);
        }
    }
}

pub fn weak_enemy_behaviour(entity: &mut Entity, map: &Map, player_position: Point) {
    entity.gravity(map, 1.0);
    entity.normal_enemy_movement(map, player_position, MOVE_SPEED * 0.5);

    if entity.action_cooldown > 0 {
        entity.action_cooldown -= 1;
        return;
    }

    if entity.did_damage {
        entity.action_cooldown = MELEE_ENEMY_ATTACK_COOLDOWN;
        entity.did_damage = false;
        // attack animation
        if let Some((action_texture_id, action_cooldown)) = entity.entity_type.get_action_animation_data() {
            entity.sprite.switch_sprite_for_action(action_texture_id, action_cooldown);
        }
    }
}

pub fn death_behaviour(entity: &mut Entity, renderer_data: &RendererData) -> Vec<EntityEvent> {
    let mut events = Vec::new();
    match entity.entity_type {
        RedBarrel => {
            let explosion = generate_entities(
                ExplodedRedBarrel,
                entity.mover.position,
                entity.mover.foot_level,
                0.0,
                renderer_data,
                0.0,
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
