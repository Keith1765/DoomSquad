//use Geogebra API to parse map from Geogebra

use crate::game::map::{self, Point, ShapeType};
use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;

pub struct GeogebraPoint {
    pub label: String,
    pub x: f64,
    pub y: f64,
}
pub struct GeogebraPolygon {
    pub label: String,
    pub vertices: Vec<String>,
    pub segments: Vec<String>,
}

pub fn parse_map() -> Result<map::Map> {
    let ggb_path = "src/parser/geogebra_only_polygone.xml";

    Ok(reading_attr_from_ggb(ggb_path).unwrap())
}

fn reading_attr_from_ggb(path: &str) -> Result<map::Map> {
    //XML laden
    let mut file = File::open(path)?;
    let mut xml_content = String::new();
    file.read_to_string(&mut xml_content)?;

    let mut reader = Reader::from_str(&xml_content);
    let mut buf = Vec::new();

    let mut point_list: Vec<GeogebraPoint> = Vec::new();
    let mut polygon_list: Vec<GeogebraPolygon> = Vec::new();

    let mut map = map::Map {
        id: 0,
        wall_sides: Vec::new(),
        wall_shapes: Vec::new(),
        block_sides: Vec::new(),
        block_shapes: Vec::new(),
        entities: Vec::new(),
        side_count: 0,
        shape_count: 0,
    };

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
                    Some("point") => read_point(&mut reader, &mut buf, &label, &mut point_list)?,
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
                        read_polygon(&mut reader, &mut buf, "Polygon", &mut polygon_list)?;
                    }
                    _ => {}
                }
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }
    //TODO Points und Segmente kombinieren
    // println!("Found {} polygons", polygon_list.len());
    // for x in polygon_list {
    //     println!("Polygon: Label: {}, Vertices: {:?}, Segments: {:?}", x.label, x.vertices, x.segments);
    // }
    // println!("Found {} points", point_list.len());
    // for x in point_list {
    //     println!("Point: Label: {}, X: {}, Y: {}", x.label, x.x, x.y);
    // }
    for x in polygon_list {
        for vertex in x.vertices {
            if let Some(point) = point_list.iter().find(|p| p.label == vertex) {
                let mut input_list_of_points: Vec<Point> = Vec::new();

                println!(
                    "Polygon {} has vertex {} at ({}, {})",
                    &x.label, &vertex, &point.x, &point.y
                );
                print!(
                    "Adding point to map at scaled position: ({}, {})\n",
                    point.x * 100.0,
                    point.y * 100.0
                );
                input_list_of_points.push(Point {
                    x: point.x * 100.0,
                    y: point.y * 100.0,
                });
                map.add_shape_from_points(
                    input_list_of_points.clone(),
                    map::ShapeType::Block,
                    0.0,
                    10.0,
                    0xFFFFFF,
                    0xAAAAAA,
                    vec![0; input_list_of_points.len()],
                );
            } else {
                println!(
                    "Vertex {} of Polygon {} not found in points list",
                    vertex, x.label
                );
            }
        }
    }
    Ok(map)
}

fn read_point(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    name: &str,
    point_list: &mut Vec<GeogebraPoint>,
) -> Result<()> {
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
        let point = GeogebraPoint {
            label: name.to_string(),
            x,
            y,
        };
        point_list.push(point);
    }
    Ok(())
}

//TODO Segment struc erstellen und in vec speichern
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

fn read_polygon(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    name: &str,
    polygon_list: &mut Vec<GeogebraPolygon>,
) -> Result<()> {
    let mut name = name.to_string();
    let mut vertices: Vec<String> = Vec::new();
    let mut segments: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"input" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    vertices.push(attr.unescape_value()?.to_string());
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == b"output" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    segments.push(attr.unescape_value()?.to_string())
                }
            }

            Event::End(ref e) if e.name().as_ref() == b"command" => {
                break;
            }

            _ => {}
        }
        buf.clear();
    }
    if segments.is_empty() {
        return Ok(());
    }
    let first_segment = segments.remove(0);
    let geogebrapolygon = GeogebraPolygon {
        label: first_segment.clone(),
        vertices,
        segments: segments,
    };

    polygon_list.push(geogebrapolygon);
    Ok(())
}
