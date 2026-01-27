use std::{collections::HashMap, fs, path::Path};

use image::ImageReader;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Texture {
    pub fn get_column(&self, u: usize) -> Option<Vec<u32>> {
        if u >= self.width {
            return None;
        }

        let mut column = Vec::with_capacity(self.height);
        for v in 1..self.height+1 {
            // give column in form bottom->top, not top->bottom
            if let Some(pixel) = self.pixels.get((self.height - v) * self.width + u) {
                column.push(*pixel);
            }
        }
        Some(column)
    }
}

pub fn load_textures() -> Option<HashMap<usize, Texture>> {
    let mut textures = HashMap::new();

    let mut texture_id: usize = 0; // TODO find a more elegant solution
    for entry in fs::read_dir(Path::new("./assets/textures")).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        // TODO make whole function not fail when one file doesnt work
        let image_buffer = ImageReader::open(&path)
            .ok()?
            .decode()
            .ok()?
            .as_rgb8()?
            .clone();
        let (width, height) = image_buffer.dimensions();

        let mut pixels: Vec<u32> = Vec::with_capacity((width * height) as usize);

        for p in image_buffer.pixels() {
            let r: u32 = p.0[0] as u32;
            let g: u32 = p.0[1] as u32;
            let b: u32 = p.0[2] as u32;
            let a: u32 = 255; // no  transparent textures implemented, but our format requires an alpha value

            let pixel_color: u32 = (a << 24) | (r << 16) | (g << 8) | b;

            pixels.push(pixel_color);
        }
        let texture = Texture {
            width: width as usize,
            height: height as usize,
            pixels,
        };
        textures.insert(texture_id, texture);
        texture_id += 1;
    }
    Some(textures)
}

#[test]
fn test_texture_load() {
    assert!(!load_textures().is_some());
}
