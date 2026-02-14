use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;

pub fn parse_entitties(path: String) -> Result<()> {
    read_entitties_from_file(path)
}

pub fn read_entitties_from_file(path: String) -> Result<()> {
    let mut file = File::open(path)?;
    let mut xml_contents = String::new();
    file.read_to_string(&mut xml_contents)?;

    let mut reader = Reader::from_str(&xml_contents);
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

fn read_point(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    name: &str,
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
        println!("Read point: {} with x: {:?}, y: {:?}", name, x, y);
        buf.clear();
    }

    Ok(())
}