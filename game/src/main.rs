#![allow(dead_code)]
#![allow(unreachable_code)]

mod audio;
mod game;
mod parser;
mod render;

use crate::audio::Audio;
use crate::render::{RendererData, render_init};
use std::f64::consts::PI;
use std::time::{Instant, Duration};

use crate::parser::entities_parser::*;
use crate::parser::map_parser::*;

use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use crate::game::interactables::*;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;
const TARGET_FPS: usize = 30;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x444444;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.005;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;
const SURFACE_DEFAULT_COLOR: u32 = 0xffff00;
const AUDIO_ENABLED: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // fps count setup
    let target_frame_duration = Duration::from_secs_f64(1.0 / TARGET_FPS as f64);
    let mut next_frame_time = Instant::now() + target_frame_duration;
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_value = 0.0;

    // winit setup
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    
    // creates window safely
    let window = WindowBuilder::new()
        .with_title("game")
        .with_inner_size(LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64))
        .build(&event_loop)?;

    window.set_cursor_visible(false); // hide mouse

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture)?
    };

    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let renderer_data: RendererData = render_init(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        HORIZONTAL_FOV,
        BACKGROUND_COLOR,
        DISTANCE_DARKNESS_COEFFICIENT,
        WALL_DEFAULT_COLOR,
        BLOCK_DEFAULT_COLOR,
        SURFACE_DEFAULT_COLOR,
    );
    let mut game = game::Game::new_test_game(&renderer_data);

    //TODO TEST
    let map = parse_map("assets/maps/ggb/geogebra_test_map_with_jump+run+entities.ggb".to_string());
    if let Ok(map) = map {
        game.map = map;
    } else {
        return Err("Error parsing map".into());
    }


    let mut audio: Option<Audio> = None;
    if AUDIO_ENABLED {
        audio = Audio::new().ok();

        if let Some(a) = audio.as_mut() {
            let _ = a.load_sfx("step", "assets/audio/step.wav");
            let _ = a.load_sfx("jump", "assets/audio/jump.wav");
            let _ = a.play_music_loop("assets/audio/music.wav", 0.6);
        }
    }

    let mut prev_keys = (false, false, false, false, false); // (W, A, S, D, Space)

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // Handle resize properly
        if let Event::WindowEvent { event: winit::event::WindowEvent::Resized(size), .. } = &event {
            if let Err(err) = pixels.resize_surface(size.width, size.height) {
                eprintln!("pixels.resize_surface error: {err}");
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Exit on Escape or window close
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            let now = Instant::now();
            if now >= next_frame_time {
                let (_, _, _, _, prev_space) = prev_keys;

                let cur_w = input.key_held(VirtualKeyCode::W);
                let cur_a = input.key_held(VirtualKeyCode::A);
                let cur_s = input.key_held(VirtualKeyCode::S);
                let cur_d = input.key_held(VirtualKeyCode::D);
                let cur_space = input.key_held(VirtualKeyCode::Space);

                if let Some(a) = &mut audio {
                    if cur_w || cur_a || cur_s || cur_d {
                        a.play_step();
                    }
                    if cur_space && !prev_space {
                        a.play_sfx("jump");
                    }
                }

                prev_keys = (cur_w, cur_a, cur_s, cur_d, cur_space);

                // Update game state
                game.update(&input, &renderer_data);
                
                // Draw to our internal buffer first (keeping your exact rendering logic)
                render::draw_screen(&mut buffer, &renderer_data, &game);

                // fps calc
                frame_count += 1;
                let elapsed = last_time.elapsed().as_secs_f32();

                if elapsed >= 1.0 {
                    fps_value = frame_count as f32 / elapsed;
                    frame_count = 0;
                    last_time = Instant::now();
                    window.set_title(&format!("My Window | FPS: {:.1}", fps_value));
                }

                // Copy internal u32 buffer to pixels u8 RGBA buffer safely
                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let rgba = buffer[i];
                    pixel[0] = (rgba >> 16) as u8; // R
                    pixel[1] = (rgba >> 8) as u8;  // G
                    pixel[2] = rgba as u8;         // B
                    pixel[3] = 255;                // A
                }

                if let Err(e) = pixels.render() {
                    eprintln!("failed to update the window: {e}");
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                next_frame_time = now + target_frame_duration;
            }
        }
    });

    Ok(())
}