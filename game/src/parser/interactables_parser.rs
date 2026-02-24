use crate::game::interactables::*;
use crate::game::map::Point;
use crate::render::RendererData;

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;
use zip::read::ZipFile;

use std::fs::File;
use std::io::Read;

use crate::parser::map_parser::SCALING_FACTOR;

pub fn parse_interactables(
    path: String,
    renderer_data: &RendererData,
) -> Result<Vec<Interactable>> {
    // ZIP-Archiv öffnen
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut xml_file = archive.by_name("geogebra.xml")?;

    // XML-Inhalt lesen
    read_interactables_from_file(&mut xml_file, renderer_data)
}

pub fn read_interactables_from_file(
    xml_file: &mut ZipFile<File>,
    renderer_data: &RendererData,
) -> Result<Vec<Interactable>> {
    let mut xml_contents = String::new();
    xml_file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
    let mut buf = Vec::new();

    let mut interactable_vector: Vec<Interactable> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"element" => {
                let mut element_type = None; //needed for later
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
                    match interactable_string_type[1].as_str() {
                        //todo
                        "Button" => read_values_for_button(
                            &mut reader,
                            &mut buf,
                            &mut interactable_vector,
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
    Ok(interactable_vector)
}
fn read_values_for_button(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    interactable_vector: &mut Vec<Interactable>,
    renderer_data: &RendererData,
    interactable_string_type: Vec<String>,
) -> Result<()> {
    let mut x = None;
    let mut y = None;
    let mut floor_level = None;
    let mut parameter_1 = None;
    let mut parameter_2 = None;
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
                        b"g" => parameter_1 = Some(attr.unescape_value()?.parse::<f64>()?),
                        b"b" => parameter_2 = Some(attr.unescape_value()?.parse::<f64>()?),
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
        "Map" => {
             let interactable = Interactable::new(
                InteractableType::Button(ButtonType::Map),
                Point {
                    x: x.unwrap() * SCALING_FACTOR,
                    y: y.unwrap() * SCALING_FACTOR,
                },
                floor_level.unwrap(),
                parameter_1.unwrap(),
                parameter_2.unwrap(),
                14,
                &renderer_data,
            )
            .unwrap();
            interactable_vector.push(interactable);
        }
        "Spawner" => {
             let interactable = Interactable::new(
                InteractableType::Button(ButtonType::Spawner),
                Point {
                    x: x.unwrap() * SCALING_FACTOR,
                    y: y.unwrap() * SCALING_FACTOR,
                },
                floor_level.unwrap(),
                parameter_1.unwrap(),
                parameter_2.unwrap(),
                14,
                &renderer_data,
            )
            .unwrap();
            interactable_vector.push(interactable);
        }
        "Heal" => {}
        _ => {}
    }
    Ok(())
}

pub fn split_string_uppercase(input: &str) -> Vec<String> {
    let cleaned = if let Some(prefix) = input.strip_suffix('}') {
        if let Some(pos) = prefix.rfind("_{") {
            let number_part = &prefix[pos + 2..];
            if number_part.chars().all(|c| c.is_ascii_digit()) {
                &prefix[..pos]
            } else {
                input
            }
        } else {
            input
        }
    } else {
        input
    };
    let mut parts = Vec::new();
    let mut current = String::new();
    for c in cleaned.chars() {
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
