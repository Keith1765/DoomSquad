use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;
use std::str;
use crate::game::entities::EntityType;
use crate::game::map::{Point};

pub struct EntityInParser {
    pub if_player: bool,     //true if player, false if enemy
    pub point: Point,        //pos from the point
    pub floor_level: f64,      //r from rgba-value
    pub facing_direction: f64, //g from rgba-value
    pub enemy_type: EntityType,       //b from rgba-value
}
     
pub fn parse_entitties(path: String) -> Result<()> {
    read_entitties_from_file(path)
}

pub fn read_entitties_from_file(path: String) -> Result<()> {
    let mut file = File::open(path)?;
    let mut xml_contents = String::new();
    file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
    let mut buf = Vec::new();

    let mut entities: Vec<EntityInParser> = Vec::new();
    

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
                        read_point(&mut reader, &mut buf, label, &mut entities)?
                    }
                    (Some("point"), label) if label.starts_with("Enemy") => {
                        read_point(&mut reader, &mut buf, label, &mut entities)?
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }
    for entity in entities {
        println!(
            "EntityInParser: if_player = {}, x = {}, y = {}, floor_level = {}, facing_direction = {}, enemy_type = {}",
            entity.if_player, entity.point.x, entity.point.y, entity.floor_level, entity.facing_direction, entity.enemy_type
        );
    }
    Ok(())
}

fn read_point(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str, entities: &mut Vec<EntityInParser>) -> Result<()> {
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
    //TODO formating vaules correctly to Entity, for now just print them
    // let entity= Entity::new(
    //     Point { x: x.unwrap_or(0.0), y: y.unwrap_or(0.0) },
    //     floor_level.unwrap_or(0.0),
    //     0.0, //TODO define in entities,
    //     facing_direction.unwrap_or(0.0),
    //     1, //TODO define in entities
    //     &RendererData: //TODO remove this, only needed for sprite size, find better solution
    //     enemy_type_num.map_or(EntityType::Dummy, |id| {
    //         match id {
    //             1 => EntityType::Bullet,
    //             2 => EntityType::RedBarrel,
    //             3 => EntityType::MeleeEnemy,
    //             4 => EntityType::RangedEnemy,
    //             5 => EntityType::SummonerEnemy,
    //             6 => EntityType::SummonedEnemy,
    //             _ => EntityType::Dummy,
    //         }
    //     })
    // );
    let entity_in_parser = EntityInParser {
        if_player: name.starts_with("Player"),
        point: Point { x: x.unwrap_or(0.0), y: y.unwrap_or(0.0) },
        floor_level: floor_level.unwrap_or(0.0),
        facing_direction: facing_direction.unwrap_or(0.0),
        enemy_type: enemy_type_num.map_or(EntityType::Dummy, |id| {
            match id {
                1 => EntityType::Bullet,
                2 => EntityType::RedBarrel,
                3 => EntityType::MeleeEnemy,
                4 => EntityType::RangedEnemy,
                5 => EntityType::SummonerEnemy,
                6 => EntityType::SummonedEnemy,
                _ => EntityType::Dummy,

            }
        }),
    };
    entities.push(entity_in_parser);
    Ok(())
}
