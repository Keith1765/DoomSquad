use crate::game::entities::{Entity, EntityType};
use crate::game::generate_entities::*;
use crate::game::map::Point;
use crate::parser::map_parser::SCALING_FACTOR;
use crate::render::RendererData;
use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;
use std::str;

pub struct ParserEntity {
    pub if_player: bool,     //true if player, false if enemy
    pub x: f64,            //pos from the point
    pub y: f64,            //pos from the point
    floor_level: f64,      //r from rgba-value
    facing_direction: f64, //g from rgba-value
    enemy_type: i32,       //b from rgba-value
}

pub fn parse_entities(path: String, renderer_data: &RendererData) -> Result<Vec<Entity>> {
    read_entities_from_file(path, renderer_data)
}

pub fn read_entities_from_file(path: String, renderer_data: &RendererData) -> Result<Vec<Entity>> {
    let mut file = File::open(path)?;
    let mut xml_contents = String::new();
    file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
    let mut buf = Vec::new();

    let mut entities: Vec<Entity> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"element" => {
                let mut element_type = None;
                let mut label = "unnamed".to_string();

                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"type" => element_type = Some(attr.unescape_value()?.to_string()),
                        b"label" => label = attr.unescape_value()?.to_string(),
                        _ => {}
                    }
                }

                match (element_type.as_deref(), label.as_str()) {
                    (Some("point"), label) if label.starts_with("Player") => {
                        read_values_for_player_from_point()? //TODO
                    }
                    (Some("point"), label) if label.starts_with("Enemy") => {
                        read_point(
                            &mut reader,
                            &mut buf,
                            label,
                            &mut entities,
                            renderer_data,
                        )?
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(entities)
}

fn read_point(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str, entities: &mut Vec<Entity>, renderer_data: &RendererData) -> Result<()> {
    let mut x = None;
    let mut y = None;
    let mut floor_level = None;
    let mut facing_direction = None;
    let mut enemy_type_num = None;
    loop {
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"coords" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"x" => x = Some(attr.unescape_value()?.parse::<f64>()?),
                        b"y" => y = Some(attr.unescape_value()?.parse::<f64>()?),
                        _ => {}
                    }
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == b"objColor" => {
                for attr in e.attributes() {
                    let attr = attr?;

                    match attr.key.as_ref() {
                        b"r" => floor_level = Some(attr.unescape_value()?.parse::<f64>()?),
                        b"g" => facing_direction = Some(attr.unescape_value()?.parse::<f64>()?),
                        b"b" => enemy_type_num = Some(attr.unescape_value()?.parse::<i32>()?),
                        //b"alpha" => idk = Some(attr.unescape_value()?.parse::<u8>()?), //left for later use
                        _ => {}
                    }
                }
            }
            Event::End(ref e) if e.name().as_ref() == b"element" => break,
            _ => {}
        }
        buf.clear();
    }
    let entity_type = enemy_type_num.map_or(EntityType::Dummy, |id| match id {
        1 => EntityType::PlayerBullet,
        2 => EntityType::RedBarrel,
        3 => EntityType::MeleeEnemy,
        4 => EntityType::RangedEnemy,
        5 => EntityType::SummonerEnemy,
        6 => EntityType::WeakEnemy,
        7 => EntityType::EnemyArrow,
        8 => EntityType::PlayerArrow,
        9 => EntityType::Archer,
        10 => EntityType::Button,
        _ => EntityType::Dummy,
    });

    let entities_pushing = generate_entities(
        entity_type,
        Point {
            x: x.unwrap() * SCALING_FACTOR,
            y: y.unwrap() * SCALING_FACTOR,
        },
        floor_level.unwrap(),
        facing_direction.unwrap(),
        renderer_data,
    );

    entities.push(entities_pushing);
    Ok(())
}

fn read_values_for_player_from_point() -> Result<()> {
    //TODO if needed, implement player parsing, but for now we can just spawn player at start pos
    Ok(())
}
