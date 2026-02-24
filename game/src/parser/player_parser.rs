use crate::game::map::Point;
use crate::parser::map_parser::SCALING_FACTOR;
use crate::render::RendererData;
use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
use zip::read::ZipFile;

pub fn parse_player_position(path: String, renderer_data: &RendererData) -> Result<Point> {
    // ZIP-Archiv öffnen
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut xml_file = archive.by_name("geogebra.xml")?;

    // XML-Inhalt lesen
    let player_point = read_player_from_file(&mut xml_file, renderer_data)?;
    Ok(player_point)
}

pub fn read_player_from_file(
    xml_file: &mut ZipFile<File>,
    _renderer_data: &RendererData,
) -> Result<Point> {
    let mut xml_contents = String::new();
    xml_file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"element" => {
                let mut label = None;

                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"label" {
                        label = Some(attr.unescape_value()?.to_string());
                    }
                }

                if let Some(label) = label {
                    if label.starts_with("Player") {
                        return read_coords_for_player(&mut reader, &mut buf);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Err(anyhow::anyhow!("no player in XML"))
}

fn read_coords_for_player(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<Point> {
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
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(Point {
        x: x.unwrap() * SCALING_FACTOR,
        y: y.unwrap() * SCALING_FACTOR,
    })
}