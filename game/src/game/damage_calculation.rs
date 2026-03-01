
use crate::audio::audio_handler::Audio;
use crate::game::entities::EntityType::{
    EnemyArrow, EnemyBullet, ExplodedRedBarrel, MeleeEnemy, PlayerArrow, PlayerBullet, WeakEnemy,
};
use crate::game::entities::{
    ARROW_DMG, BULLET_DMG, MEELE_ENEMY_DMG, MELEE_ENEMY_RANGE, RED_BARREL_DMG,
    WEAK_ENEMY_MULTIPLICATOR,
};
use crate::game::gamestate::{Game, INTERACTABLE_RANGE};

pub fn damage_check(game_state: &mut Game, audio: &mut Audio) {
    //DMG calc for entities

    //update grid
    game_state.map_grid.update(&game_state.entities);

    //let mut projectile_that_hit = Vec::new();
    game_state.projectile_that_hit.clear();

    //damage calc for enemies
    for i in 0..game_state.entities.len() {
        if (game_state.entities[i].entity_type != PlayerBullet)
            && (game_state.entities[i].entity_type != PlayerArrow)
            && (game_state.entities[i].entity_type != ExplodedRedBarrel)
        {
            continue;
        }

        let projectile_position = game_state.entities[i].mover.position;

        //get all entities from neighbouring cells
        let neighbours = game_state.map_grid.get_neighbours(projectile_position);

        for j in neighbours {
            let entities_immune_to_damage = matches!(
                game_state.entities[j].entity_type,
                PlayerBullet | PlayerArrow | ExplodedRedBarrel
            );

            // println!("proximity: {}", game_state.entities[j].entity_type);
            // println!("proximity: {}", game_state.entities[j].mover.view_level);

            //no self collision
            if i == j {
                continue;
            }
            //no dmg calc for immune entities
            if entities_immune_to_damage {
                continue;
            }

            let distance_to_bullet =
                projectile_position.distance_to(&game_state.entities[j].mover.position);

            //check for verticall hitbox overlap
            if distance_to_bullet <= game_state.entities[j].size + game_state.entities[i].size {
                // println!("vertical: {}", game_state.entities[j].entity_type);
                //check if horizontal hitbox overlap
                let shooting_height = game_state.entities[i].mover.foot_level; //foot lvl because these are bullets
                let entity_bottom = game_state.entities[j].mover.foot_level;
                let entity_top = game_state.entities[j].mover.height + entity_bottom;

                if shooting_height <= entity_top && shooting_height >= entity_bottom {
                    // println!("horizontal: {}", game_state.entities[j].entity_type);
                    let damage = match game_state.entities[i].entity_type {
                        PlayerBullet => BULLET_DMG,
                        PlayerArrow => ARROW_DMG,
                        ExplodedRedBarrel => RED_BARREL_DMG,
                        _ => 0.0,
                    };
                    //DAMAGE THAT BITCH
                    game_state.entities[j].hp -= damage;

                    if damage > 0.0{
                        audio.play_sfx_distance_scaled(
                        "hit",
                        1.0,
                        game_state.player.mover.position,
                        game_state.entities[j].mover.position
                    );
                    }

                    //projectiles go brr
                    if game_state.entities[i].entity_type != ExplodedRedBarrel {
                        game_state.projectile_that_hit.push(i);
                    }
                    if game_state.entities[i].entity_type != ExplodedRedBarrel {
                        break; //cause no entity penetration
                    }
                }
            }
        }
    }

    //calc damage to player
    let player_position = game_state.player.mover.position;

    //get all entities from neighbouring cells
    let neighbours = game_state.map_grid.get_neighbours(player_position);

    for j in neighbours {
        let distance_to_player =
            player_position.distance_to(&game_state.entities[j].mover.position);

        //check if vertical overlap of hitbox
        if (distance_to_player
            <= game_state.entities[j].size + game_state.player.size + MELEE_ENEMY_RANGE)
            && !game_state.player.godmode // invincible in godmode
        {
            //check if horizontal overlap of hitbox
            let shooting_height = game_state.entities[j].mover.view_level;
            let player_bottom = game_state.player.mover.foot_level;
            let player_top = game_state.player.mover.height + player_bottom;

            if shooting_height <= player_top && shooting_height >= player_bottom {
                let damage = match game_state.entities[j].entity_type {
                    EnemyBullet => BULLET_DMG,
                    EnemyArrow => ARROW_DMG,
                    ExplodedRedBarrel => match game_state.entities[j].did_damage {
                        true => 0.0,
                        false => RED_BARREL_DMG,
                    },
                    MeleeEnemy => match game_state.entities[j].did_damage {
                        true => 0.0,
                        false => MEELE_ENEMY_DMG,
                    },
                    WeakEnemy => match game_state.entities[j].did_damage {
                        true => 0.0,
                        false => MEELE_ENEMY_DMG * WEAK_ENEMY_MULTIPLICATOR,
                    },
                    _ => 0.0,
                };
                //DAMAGE THAT BITCH
                game_state.player.hp -= damage;

                if damage > 0.0 {
                    match game_state.entities[j].entity_type {
                        MeleeEnemy => game_state.entities[j].did_damage = true,
                        WeakEnemy => game_state.entities[j].did_damage = true,
                        _ => {}
                    }
                    audio.play_sfx(
                        "hit",
                        1.0,
                    );
                }

                //bullet go brr
                if matches!(game_state.entities[j].entity_type, EnemyArrow | EnemyBullet) {
                    game_state.projectile_that_hit.push(j);
                }

                break; //cause no entity penetration
            }
        }
    }

    //delete all bullets that hit
    for i in game_state.projectile_that_hit.clone() {
        game_state.entities[i].hp = 0.0; //o7
    }
    //Interactables

    for _i in 0..game_state.interactables.len() {
        game_state.map_grid.update_interactables(&game_state.interactables);

        for interactable in &mut game_state.interactables {
            interactable.player_in_range = false;
        }

        let player_position = game_state.player.mover.position;

        let neighbours = game_state.map_grid.get_interactable_neighbours(player_position);

        for j in neighbours {
            let interactable_position = game_state.interactables[j].mover.position;
            let distance_from_player_to_interactable =
                player_position.distance_to(&interactable_position);
            if distance_from_player_to_interactable <= INTERACTABLE_RANGE {
                game_state.interactables[j].player_in_range = true;
            }
        }
    }
    
}
