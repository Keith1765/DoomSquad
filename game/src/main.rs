#![allow(dead_code)]

mod audio;
mod game;
mod render;

use crate::audio::Audio;
use crate::render::{RendererData, render_init};
use minifb::{Key, Window, WindowOptions};
use std::f64::consts::PI;
use std::time::Instant;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;
const TARGET_FPS: usize = 60;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x222222;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.025;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;

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
        BLOCK_DEFAULT_COLOR,
    );

    let mut audio = Audio::new().ok();

    if let Some(a) = audio.as_mut() {
        let _ = a.load_sfx("step", "assets/audio/step.wav");
        let _ = a.load_sfx("jump", "assets/audio/jump.wav");
        let _ = a.play_music_loop("assets/audio/music.wav", 0.6);
    }

    let mut prev_keys = (false, false, false, false, false); // (W, A, S, D, Space)

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (_, _, _, _, prev_space) = prev_keys;

        let cur_w = window.is_key_down(Key::W);
        let cur_a = window.is_key_down(Key::A);
        let cur_s = window.is_key_down(Key::S);
        let cur_d = window.is_key_down(Key::D);
        let cur_space = window.is_key_down(Key::Space);

        if let Some(a) = &mut audio {
            // call this for any movement key pressed
            if cur_w || cur_a || cur_s || cur_d {
                a.play_step(); // step sound with cooldown
            }

            // jump SFX
            if cur_space && !prev_space {
                a.play_sfx("jump");
            }
        }

        prev_keys = (cur_w, cur_a, cur_s, cur_d, cur_space);

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
