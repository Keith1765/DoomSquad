//use Geogebra API to parse map from Geogebra

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;

pub fn parse_map() -> Result<()> {
    let ggb_path = "src/parser/map1.ggb";
    reading_attr_from_ggb(ggb_path)?;
    Ok(())
}

fn read_ggb_as_string(path: &str) -> Result<String> {
    let file = File::open(path).with_context(|| format!("Failed to open {}", path))?;

    let mut archive = ZipArchive::new(file).context("File is not a valid .ggb (zip) archive")?;

    // GeoGebra always stores data in "geogebra.xml"
    let mut xml_file = archive
        .by_name("geogebra.xml")
        .context("geogebra.xml not found in .ggb file")?;

    let mut contents = String::new();
    xml_file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn reading_attr_from_ggb(path: &str) -> Result<()> {
    let file = File::open(path).with_context(|| format!("Failed to open {}", path))?;

    let mut archive = ZipArchive::new(file).context("File is not a valid .ggb (zip) archive")?;

    // geogebra.xml aus dem ZIP holen
    let mut xml_file = archive
        .by_name("geogebra.xml")
        .context("geogebra.xml not found in .ggb file")?;

    // XML in String laden
    let mut xml_content = String::new();
    xml_file.read_to_string(&mut xml_content)?;

    // XML-Reader vorbereiten
    let mut reader = Reader::from_str(&xml_content);

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"element" => {
                let mut is_point = false;
                let mut name = None;

                for attr in e.attributes() {
                    let attr = attr?;
                    let key = std::str::from_utf8(attr.key.as_ref())?;
                    let value = attr.unescape_value()?;

                    if key == "type" && value == "point" {
                        is_point = true; //
                    }
                    if key == "label" {
                        name = Some(value.to_string());
                    }
                }
                if is_point {
                    let mut x = None;
                    let mut y = None;
                    loop {
                        match reader.read_event_into(&mut buf)? {
                            Event::Empty(ref coords) if coords.name().as_ref() == b"coords" => {
                                for attr in coords.attributes() {
                                    let attr = attr?;
                                    match attr.key.as_ref() {
                                        b"x" => x = Some(attr.unescape_value()?.parse::<f64>()?),
                                        b"y" => y = Some(attr.unescape_value()?.parse::<f64>()?),
                                        _ => {}
                                    }
                                }
                                break;
                            }
                            Event::End(ref end) if end.name().as_ref() == b"element" => break,
                            _ => {}
                        }
                    }
                    println!(
                        "Punkt {}: ({:?}, {:?})",
                        name.unwrap_or("unnamed".to_string()),
                        x.unwrap(),
                        y.unwrap()
                    );
                }
            }
            //DAtein end
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}
