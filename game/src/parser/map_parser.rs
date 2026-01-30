//use Geogebra API to parse map from Geogebra

use anyhow::{Context, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::{BufReader, Read};

pub fn parse_map() -> Result<()> {
    let ggb_path = "src/parser/geogebra.xml";
    reading_attr_from_ggb(ggb_path)?;
    Ok(())
}

fn reading_attr_from_ggb(path: &str) -> Result<()> {
    //XML laden
    let mut file = File::open(path)?;
    let mut xml_content = String::new();
    file.read_to_string(&mut xml_content)?;

    let mut reader = Reader::from_str(&xml_content);

    let mut buf = Vec::new();

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

                match element_type.as_deref() {
                    Some("point") => read_point(&mut reader, &mut buf, &label)?,
                    Some("segment") => read_segment(&mut reader, &mut buf, &label)?,
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

fn read_point(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str) -> Result<()> {
    let mut x = None;
    let mut y = None;

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
            Event::End(ref e) if e.name().as_ref() == b"element" => break,
            _ => {}
        }
        buf.clear();
    }

    if let (Some(x), Some(y)) = (x, y) {
        println!("Punkt {}: ({}, {})", name, x, y);
    }

    Ok(())
}

fn read_segment(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str) -> Result<()> {
    let mut p1 = "unnamed".to_string();
    let mut p2 = "unnamed".to_string();

    loop {
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"segment" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"point1" => p1 = attr.unescape_value()?.to_string(),
                        b"point2" => p2 = attr.unescape_value()?.to_string(),
                        _ => {}
                    }
                }
            }
            Event::End(ref e) if e.name().as_ref() == b"element" => break,
            _ => {}
        }
        buf.clear();
    }

    println!("Segment {} zwischen {} und {}", name, p1, p2);
    Ok(())
}
