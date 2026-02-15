use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;
use std::str;

pub struct Entity {
    pub if_player: bool,     //true if player, false if enemy
    pub x: f64,            //pos from the point
    pub y: f64,            //pos from the point
    floor_level: f64,      //r from rgba-value
    facing_direction: f64, //g from rgba-value
    enemy_type: i32,       //b from rgba-value
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
            "Entity: if_player = {}, x = {}, y = {}, floor_level = {}, facing_direction = {}, enemy_type = {}",
            entity.if_player, entity.x, entity.y, entity.floor_level, entity.facing_direction, entity.enemy_type
        );
    }
    Ok(())
}

fn read_point(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str, entities: &mut Vec<Entity>) -> Result<()> {
    let mut x = None;
    let mut y = None;
    let mut floor_level = None;
    let mut facing_direction = None;
    let mut enemy_type = None;
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
                        b"b" => enemy_type = Some(attr.unescape_value()?.parse::<i32>()?),
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
    let entity = Entity {
        if_player: name.starts_with("Player"),
        x: x.unwrap_or(0.0),
        y: y.unwrap_or(0.0),
        floor_level: floor_level.unwrap_or(0.0),
        facing_direction: facing_direction.unwrap_or(0.0),
        enemy_type: enemy_type.unwrap_or(0),
    };
    entities.push(entity);
    Ok(())
}
