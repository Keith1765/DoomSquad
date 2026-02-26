use crate::game::entities::{
    ARROW_COOLDOWN, ARROW_SPEED, BULLET_SPEED, ENEMY_HP, ENEMY_SIZE, ENTITY_DEFAULT_VIEW_HEIGHT, Entity, EntityEvent, MELEE_ENEMY_ATTACK_COOLDOWN, SHOOTING_COOLDOWN, SUMMONING_COOLDOWN
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
    entity.hp -= 0.25; //prevent long projectile life
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(BULLET_SPEED, 0.0, map, false);
    entity.mover.foot_level -= entity.vertical_aim*BULLET_FLIGHT_COEFICIENT; //adjust trajectory based on player aim

    //if projectiles hits floor or hit a wall and stopped moving, we delete it for performance issues
    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}


pub fn enemy_bullet_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25; //prevent long projectile life
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(BULLET_SPEED, 0.0, map, false);

    //if projectiles hits floor or hit a wall and stopped moving, we delete it for performance issues
    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}

pub fn player_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25; //prevent long projectile life
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.mover.foot_level -= entity.vertical_aim*ARROW_FLIGHT_COEFICIENT;
    entity.gravity(map, 0.1);

    //if projectiles hits floor or hit a wall and stopped moving, we delete it for performance issues
    if (entity.mover.foot_level <= entity.mover.floor_level) || (temp_position == entity.mover.position) {
        entity.hp = 0.0;
    }
}
//atm the same as player bullets
pub fn enemy_arrow_behaviour(entity: &mut Entity, map: &Map) {
    entity.hp -= 0.25; //prevent long projectile life
    let temp_position = entity.mover.position; //safe last pos before moving, if doesnt change after move, delete entity
    entity.mover.step(ARROW_SPEED, 0.0, map, false);
    entity.gravity(map, 0.1);
    //if projectiles hits floor or hit a wall and stopped moving, we delete it for performance issues
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

//doesnt move, creates bullet entity and hands it over as an spawn event
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
            ENTITY_DEFAULT_VIEW_HEIGHT,
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

//doesnt move, creates arrow entity and hands it over as an spawn event
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

//doesnt move, creates weak enemy entity and hands it over as an spawn event
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

//moves towards player and deals damage with cooldown if in proximity (dmg handled by damage_calculation)
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

//melee enemy but weaker
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

//handles death behaviour of entity and creates entities for Spawn event, originally planned for more entities, rn used just for exploded barrel
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
