//use Geogebra API to parse map from Geogebra

use crate::game::map::{self, Point, ShapeType};
use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
use zip::read::ZipFile;
pub const SCALING_FACTOR: f64 = 100.0;

pub struct GeogebraPoint {
    pub label: String,
    pub x: f64,
    pub y: f64,
}
pub struct GeogebraPolygonCommand {
    pub label: String,
    pub vertices: Vec<String>,
    pub segments: Vec<String>,
}
pub struct GeogebraPolygonElement {
    pub label: String,
    pub bottom: f64,
    pub height: f64,
    pub surface_color: u32,
    texture_id: usize,
    pub shape_type: ShapeType,
}
//defing a struct for ez Variable export
pub struct GeogebraPolygone {
    label: String,
    shape_type: ShapeType, //show object ture/ false
    bottom: f64,
    height: f64,
    surface_color: u32,
    texture_id: usize,
    vertices: Vec<String>,
    segments: Vec<String>,
}

impl Default for GeogebraPolygone {
    fn default() -> Self {
        Self {
            label: String::new(),
            shape_type: ShapeType::Block,
            bottom: 0.0,
            height: 0.0,
            surface_color: 0xFFFFFF,
            texture_id: 0,
            vertices: Vec::new(),
            segments: Vec::new(),
        }
    }
}

pub fn parse_map(path: String) -> Result<map::Map> {
    // ZIP-Archiv öffnen
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut xml_file = archive.by_name("geogebra.xml")?;

    // XML-Inhalt lesen
    reading_attr_from_ggb(&mut xml_file)
}

fn reading_attr_from_ggb(xml_file: &mut ZipFile<File>) -> Result<map::Map> {
    //XML laden
    let mut xml_content = String::new();
    xml_file.read_to_string(&mut xml_content)?;
    //reader erstellen
    let mut reader = Reader::from_str(&xml_content);
    let mut buf = Vec::new();
    //list with all exportet points, segments and polygons(for the element/ command loop) from geogebra
    let mut point_list: Vec<GeogebraPoint> = Vec::new();
    let mut polygon_command_list: Vec<GeogebraPolygonCommand> = Vec::new();
    let mut polygon_element_list: Vec<GeogebraPolygonElement> = Vec::new();
    let mut geogebra_polygone_list: Vec<GeogebraPolygone> = Vec::new();

    let mut map = map::Map {
        id: 0,
        wall_sides: Vec::new(),
        wall_shapes: Vec::new(),
        block_sides: Vec::new(),
        block_shapes: Vec::new(),
        side_count: 0,
        shape_count: 0,
    };
    //first loop cheching for elements
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
                //second loop for the element types
                match element_type.as_deref() {
                    Some("point") => read_point(&mut reader, &mut buf, &label, &mut point_list)?,
                    Some("segment") => read_segment(&mut reader, &mut buf, &label)?,
                    Some("polygon") => read_polygon_element(
                        &mut reader,
                        &mut buf,
                        &label,
                        &mut polygon_element_list,
                    )?,
                    _ => {}
                }
            }
            //first loop checking for commands
            Event::Start(ref e) if e.name().as_ref() == b"command" => {
                let mut command_name = None;

                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"name" {
                        command_name = Some(attr.unescape_value()?.to_string());
                    }
                }

                if let Some("Polygon") = command_name.as_deref() {
                    read_polygon_command(
                        &mut reader,
                        &mut buf,
                        "Polygon",
                        &mut polygon_command_list,
                    )?;
                }
            }

            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }
    for p in &polygon_command_list {
        let mut input_list_of_points: Vec<Point> = Vec::new();
        for vertex in &p.vertices {
            if let Some(point) = point_list.iter().find(|p| p.label == *vertex) {
                input_list_of_points.push(Point {
                    x: point.x * SCALING_FACTOR,
                    y: point.y * SCALING_FACTOR,
                });
            }
        }
    }
    //combining the element and command list to one list with all information for the map
    for e in &polygon_element_list {
        for c in &polygon_command_list {
            if c.label == e.label {
                let geogebra_polygon = GeogebraPolygone {
                    label: e.label.clone(),
                    shape_type: e.shape_type,
                    bottom: e.bottom,
                    height: e.height,
                    surface_color: e.surface_color,
                    texture_id: e.texture_id,
                    vertices: c.vertices.clone(),
                    segments: c.segments.clone(),
                };
                geogebra_polygone_list.push(geogebra_polygon);
            }
        }
    }
    for p in geogebra_polygone_list {
        let mut input_list_of_points: Vec<Point> = Vec::new();
        for vertex in &p.vertices {
            if let Some(point) = point_list.iter().find(|pt| pt.label == *vertex) {
                input_list_of_points.push(Point {
                    x: point.x * SCALING_FACTOR,
                    y: point.y * SCALING_FACTOR,
                });
            }
        }
        map.add_shape_from_points(
            input_list_of_points.clone(),
            p.shape_type,
            p.bottom,
            p.height,
            0xAAAAAA,//not needed bc we havbe textures
            p.surface_color,
            vec![p.texture_id; input_list_of_points.len()],
        );
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

//TODO creating Segment struc
//left to implemt diffrent texture for each segment, but for now we will use the same texture for all segments
fn read_segment(_reader: &mut Reader<&[u8]>, _buf: &mut Vec<u8>, _name: &str) -> Result<()> {
    // loop {
    //     let mut r = None; //left for later use
    //     let mut g = None; //left for later use
    //     let mut b = None; //left for later use
    //     let mut alpha = None; //left for later use

    //     match reader.read_event_into(buf)? {
    //         Event::Empty(ref e) if e.name().as_ref() == b"objColor" => {
    //             for attr in e.attributes() {
    //                 let attr = attr?;

    //                 match attr.key.as_ref() {
    //                     b"r" => r = Some(attr.unescape_value()?.parse::<u8>()?), //left for later use
    //                     b"g" => g = Some(attr.unescape_value()?.parse::<u8>()?), //left for later use
    //                     b"b" => b = Some(attr.unescape_value()?.parse::<u8>()?), //left for later use
    //                     b"alpha" => alpha = Some(attr.unescape_value()?.parse::<u8>()?), //left for later use
    //                     _ => {}
    //                 }
    //             }
    //         }

    //         Event::End(ref e) if e.name().as_ref() == b"element" => break,
    //         _ => {}
    //     }

    //     buf.clear();
    // }
    Ok(())
}

//exporting info from command bc there is the information what points belong to which polygon
fn read_polygon_command(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    name: &str,
    polygon_command_list: &mut Vec<GeogebraPolygonCommand>,
) -> Result<()> {
    let _name = name.to_string(); //need for later implementation and for debuging
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
    let geogebrapolygon = GeogebraPolygonCommand {
        label: first_segment.clone(),
        vertices,
        segments,
    };

    polygon_command_list.push(geogebrapolygon);
    Ok(())
}

//reading in command bc there is info about color and visibility
//r => botom (the the polygon beginns)
//g => height (how high the polygon is)
//b => texture id (which texture the polygon has)
//alpha => surface color (the top color of polygon)
fn read_polygon_element(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    name: &str,
    polygon_element_list: &mut Vec<GeogebraPolygonElement>,
) -> Result<()> {
    let name = name.to_string();
    let mut bottom: f64 = 0.0;
    let mut height: f64 = 0.0;
    let mut surface_color: f64 = 16711680.0; //default red
    let mut texture_id: f64 = 0.0;
    let mut shape_type = ShapeType::Block;
    loop {
        match reader.read_event_into(buf)? {
            Event::Empty(ref e) if e.name().as_ref() == b"show" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"object"  {
                        if attr.unescape_value()?.as_ref() == "true" {
                            shape_type = ShapeType::Block;
                        } else {
                            shape_type = ShapeType::Wall;
                        }
                    }
                }
            }
            Event::Empty(ref e) if e.name().as_ref() == b"objColor" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"r" => bottom = attr.unescape_value()?.parse::<u8>()? as f64,
                        b"g" => height = attr.unescape_value()?.parse::<u8>()? as f64,
                        b"b" => texture_id = attr.unescape_value()?.parse::<u8>()? as f64,
                        b"alpha" => surface_color = attr.unescape_value()?.parse::<f64>()? * 100000.0, //alpha is between 0 and 1, we multiply it with 100000 to get a lager amount of coulors, because we only use the alpha value for the texture id, we can do this without losing any information
                        _ => {}
                    }
                }
                let geogebrapolygon = GeogebraPolygonElement {
                    label: name.clone(),
                    bottom,
                    height,
                    surface_color: surface_color as u32,
                    texture_id: texture_id as usize,
                    shape_type,
                };
                polygon_element_list.push(geogebrapolygon);
            }
            Event::End(ref e) if e.name().as_ref() == b"element" => break,
            _ => {}
        }
    }
    Ok(())
}
