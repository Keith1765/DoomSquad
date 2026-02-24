use crate::game::interactables::*;
use crate::game::map::Point;
use crate::render::RendererData;

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;

use std::fs::File;
use std::io::Read;

use crate::parser::map_parser::SCALING_FACTOR;

pub fn parse_interactables(
    path: String,
    renderer_data: &RendererData,
) -> Result<Vec<Interactable>> {
    read_interactables_from_file(path, renderer_data)
}

pub fn read_interactables_from_file(
    path: String,
    renderer_data: &RendererData,
) -> Result<Vec<Interactable>> {
    let mut file = File::open(path)?;
    let mut xml_contents = String::new();
    file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
    let mut buf = Vec::new();

    let mut Interactable: Vec<Interactable> = Vec::new();

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
                let interactable_string_type = split_string_uppercase(label.as_str());
                if interactable_string_type[0] == "Interactable" {
                    match (interactable_string_type[1].as_str()) {
                        //todo
                        "Button" => read_values_for_button(
                            &mut reader,
                            &mut buf,
                            &mut Interactable,
                            renderer_data,
                            interactable_string_type.clone(),
                        )?,
                        "Elevator" => {}

                        _ => {}
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }
    Ok(Interactable)
}
fn read_values_for_enemy_from_point(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    Interactable: &mut Vec<Interactable>,
    renderer_data: &RendererData,
) -> Result<()> {
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
    Ok(())
}
fn read_values_for_button(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    Interactable: &mut Vec<Interactable>,
    renderer_data: &RendererData,
    interactable_string_type: Vec<String>,
) -> Result<()> {
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
    match interactable_string_type[2].as_str() {
        "Map" => {}
        "Spawner" => {}
        _ => {}
    }
    Ok(())
}

pub fn split_string_uppercase(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        if c.is_uppercase() && !current.is_empty() {
            parts.push(current);
            current = String::new();
        }
        current.push(c);
    }

    if !current.is_empty() {
        parts.push(current);
    }
    parts
}
