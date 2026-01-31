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
        //TODO
        // match reader.read_event_into(&mut buf)? {
        //     Event::Start(ref e) if e.name().as_ref() == b"command" => {
        //     _ => {
        // }
        //TODO END
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
    loop {
        let mut r = None;
        let mut g = None;
        let mut b = None;
        let mut alpha = None;

        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"objColor" => {
                for attr in e.attributes() {
                    let attr = attr?;

                    match attr.key.as_ref() {
                        b"r" => r = Some(attr.unescape_value()?.parse::<u8>()?),
                        b"g" => g = Some(attr.unescape_value()?.parse::<u8>()?),
                        b"b" => b = Some(attr.unescape_value()?.parse::<u8>()?),
                        b"alpha" => alpha = Some(attr.unescape_value()?.parse::<u8>()?),
                        _ => {}
                    }
                }
                println!(
                    "Segment: {},Farbe: rgba({}, {}, {}, {})",
                    name,
                    r.unwrap_or(0),
                    g.unwrap_or(0),
                    b.unwrap_or(0),
                    alpha.unwrap_or(255)
                );
            }

            Event::End(ref e) if e.name().as_ref() == b"element" => break,
            _ => {}
        }

        buf.clear();
    }
    Ok(())
}
