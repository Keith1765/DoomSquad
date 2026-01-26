use std::{collections::HashMap, ffi::OsString, fs, hash::{DefaultHasher, Hash}, io, path::Path};

use image::ImageReader;

use crate::render::textures::{Texture, load_textures};

pub struct RendererData {
    pub screen_width_as_f64: f64,
    pub screen_height_as_f64: f64,
    pub horizontal_fov: f64,
    pub vertical_fov: f64,
    pub render_scale_coefficient: f64,
    pub background_color: u32,
    pub wall_default_color: u32,
    pub block_default_color: u32,
    pub surface_default_color: u32,
    pub distance_darkness_coefficient: f64,
    pub textures: HashMap<usize, Texture>,
}

pub fn render_init(
    screen_width: usize,
    screen_height: usize,
    horizontal_fov: f64,
    background_color: u32,
    distance_darkness_coefficient: f64,
    wall_default_color: u32,
    block_default_color: u32,
    surface_default_color: u32,
) -> RendererData {
    let screen_width_as_f64 = screen_width as f64;
    let screen_height_as_f64 = screen_height as f64;

    let vertical_fov: f64 =
        ((screen_height_as_f64 / screen_width as f64) * (horizontal_fov / 2.0).tan()).atan() * 2.0;

    // would be the sam with height / vertical_fov: can be used for both horizontal and vertical scaling
    let render_scale_coefficient: f64 = (screen_width as f64 / 2.0) / (horizontal_fov / 2.0).tan();

    // we accept this unwrap for now, if textures not working just crash, it's fine for now
    // TODO remove necessity for unwrap
    let textures = load_textures().unwrap();

    RendererData {
        screen_width_as_f64,
        screen_height_as_f64,
        horizontal_fov,
        vertical_fov,
        render_scale_coefficient,
        background_color,
        distance_darkness_coefficient,
        wall_default_color,
        block_default_color,
        surface_default_color,
        textures,
    }
}

