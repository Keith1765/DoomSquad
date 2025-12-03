//use Geogebra API to parse map from Geogebra 

use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
use anyhow::{Context,Result};

pub fn parse_map () -> Result<()> {
    let ggb_path = "src/parser/map1.ggb";
    let xml_string = read_ggb_as_string(ggb_path)?;
    println!("{}", xml_string);

    Ok(())
}

fn read_ggb_as_string(path: &str) -> Result<String> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open {}", path))?;

    let mut archive = ZipArchive::new(file)
        .context("File is not a valid .ggb (zip) archive")?;

    // GeoGebra always stores data in "geogebra.xml"
    let mut xml_file = archive.by_name("geogebra.xml")
        .context("geogebra.xml not found in .ggb file")?;

    let mut contents = String::new();
    xml_file.read_to_string(&mut contents)?;

    Ok(contents)
}