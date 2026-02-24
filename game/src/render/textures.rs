use std::{collections::HashMap, fs, path::Path};

use image::ImageReader;

use crate::render::RendererData;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Texture {
    // returns the column of a texture, scaled
    pub fn get_texture_column(
        &self,
        u: usize,
        onscreen_bottom: isize,
        onscreen_top: isize,
        inworld_height: f64,
        renderer_data: &RendererData,
    ) -> Option<Vec<u32>> {
        if u >= self.width {
            return None;
        }

        // TODO function still very slow: improve somehow?

        // get the unscaled column
        let mut unscaled_column = Vec::with_capacity(self.height);
        for v in 1..self.height + 1 {
            // give column in form bottom->top, not top->bottom
            if let Some(pixel) = self.pixels.get((self.height - v) * self.width + u) {
                unscaled_column.push(*pixel);
            }
        }

        let onscreen_height = onscreen_top - onscreen_bottom;
        let onscreen_height_as_f64 = onscreen_height as f64;
        let mut scaled_column: Vec<u32> = Vec::with_capacity(
            onscreen_height.clamp(0, renderer_data.screen_height_as_isize) as usize,
        );
        let inworld_onscreen_ratio = inworld_height / onscreen_height_as_f64;
        for onscreen_y in onscreen_bottom.clamp(0, renderer_data.screen_height_as_isize)
            ..onscreen_top.clamp(0, renderer_data.screen_height_as_isize)
        {
            let onscreen_y_on_tex = onscreen_y - onscreen_bottom;
            let inworld_y_on_tex = (onscreen_y_on_tex as f64 * inworld_onscreen_ratio) as usize;
            let v = inworld_y_on_tex % self.height;
            if let Some(color) = unscaled_column.get(v) {
                scaled_column.push(*color);
            }
        }

        Some(scaled_column)
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
            .to_rgba8()
            .clone();
        let (width, height) = image_buffer.dimensions();

        let mut pixels: Vec<u32> = Vec::with_capacity((width * height) as usize);

        for p in image_buffer.pixels() {
            let r: u32 = p.0[0] as u32;
            let g: u32 = p.0[1] as u32;
            let b: u32 = p.0[2] as u32;
            let a: u32 = p.0[3] as u32;
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
    assert!(load_textures().is_some());
}
