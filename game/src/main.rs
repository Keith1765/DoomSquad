#![allow(dead_code)]

mod game;
mod render;

use crate::render::{RendererData, render_init};
use minifb::{Key, MouseMode, Window, WindowOptions};
use std::f64::consts::PI;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;
const TARGET_FPS: usize = 60;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x222222;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.025;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BOTTOM_BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;
const TOP_BLOCK_DEFAULT_COLOR: u32 = 0xff0000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //for fps count
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_value = 0.0;

    //creates window Safely
    let mut window = match Window::new(
        "game",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            eprint!("failed to create Window");
            return Err(Box::new(e));
        }
    };
    window.set_cursor_visibility(false); // hide mouse 

    //to reduce CPU load by decreasing refresh rate oder so lol
    window.set_target_fps(TARGET_FPS);

    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let mut game = game::Game::new();

    let renderer_data: RendererData = render_init(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        HORIZONTAL_FOV,
        BACKGROUND_COLOR,
        DISTANCE_DARKNESS_COEFFICIENT,
        WALL_DEFAULT_COLOR,
        BOTTOM_BLOCK_DEFAULT_COLOR,
        TOP_BLOCK_DEFAULT_COLOR,
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        game.update(&window);

        render::draw(&mut buffer, &renderer_data, &mut game);

        //fps calc
        frame_count += 1;
        let elapsed = last_time.elapsed().as_secs_f32();

        if elapsed >= 1.0 {
            fps_value = frame_count as f32 / elapsed;
            frame_count = 0;
            last_time = Instant::now();

            window.set_title(&format!("My Window | FPS: {:.1}", fps_value));
        }
        //show buffer safely
        if let Err(e) = window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
    }
    Ok(())
}
