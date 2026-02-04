//use Geogebra API to parse map from Geogebra

use anyhow::{Context, Result};
use quick_xml::{Reader, name};
use quick_xml::events::Event;
use std::fs::File;
use std::io::{BufReader, Read};

pub struct GeogebraPoint {
    pub label: String,
    pub x: f64,
    pub y: f64,
}

pub fn parse_map() -> Result<()> {
    let ggb_path = "src/parser/geogebra_only_polygone.xml";
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

     let mut pointList: Vec<GeogebraPoint> = Vec::new();

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
                Some("point") => read_point(&mut reader, &mut buf, &label, &mut pointList)?,
                Some("segment") => read_segment(&mut reader, &mut buf, &label)?,
                _ => {}
            }
        }

        Event::Start(ref e) if e.name().as_ref() == b"command" => {
            let mut command_name = None;

            for attr in e.attributes() {
                let attr = attr?;
                if attr.key.as_ref() == b"name" {
                    command_name = Some(attr.unescape_value()?.to_string());
                }
            }

            match command_name.as_deref() {
                Some("Polygon") => {
                    println!("Found Polygon Command");
                    read_polygon(&mut reader, &mut buf, "Polygon")?;    
                }
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


fn read_point(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str, pointList: &mut Vec<GeogebraPoint>) -> Result<()> {
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
        let point = GeogebraPoint {label: name.to_string(), x, y };
        pointList.push(point);
    }
    for x in pointList {
        println!("Label: {}, X: {}, Y: {}", x.label, x.x, x.y);
        println!("ich bin aus der liste")
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

fn read_polygon(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>, name: &str) -> Result<()> {
    let mut name = name.to_string();
    let mut vertices: Vec<String> = Vec::new();
    loop {
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"input" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    vertices.push(attr.unescape_value()?.to_string());
                }
            }

            Event::End(ref e) if e.name().as_ref() == b"command" => {
                break;
            }

            _ => {}
        }
        // //todo
        // match reader.read_event_into(buf)? {
        //     Event::End(ref e) if e.name().as_ref() == b"output" => break,
        //     _ => {}
        // }
        // //todo end
        buf.clear();
    }

    println!("Polygon: {}, Vertices: {:?}", name, vertices);
    Ok(())
}