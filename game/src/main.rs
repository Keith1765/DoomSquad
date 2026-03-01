#![allow(dead_code)]

mod audio;
mod game;
mod parser;
mod render;
mod menu;

use crate::audio::audio_handler::Audio;
use crate::render::{RendererData, render_init};
use crate::menu::menu_handler::{Menu, AppState};
use minifb::{KeyRepeat, Key, Window, WindowOptions};
use std::f64::consts::PI;
use std::time::Instant;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;
const TARGET_FPS: usize = 30;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x444444;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.005;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;
const SURFACE_DEFAULT_COLOR: u32 = 0xffff00;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_value;

    let mut window = match Window::new("DoomSquad", SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions::default()) {
        Ok(w) => w,
        Err(e) => {
            eprint!("failed to create Window");
            return Err(Box::new(e));
        }
    };
    
    window.set_target_fps(TARGET_FPS);
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let renderer_data: RendererData = render_init(
        SCREEN_WIDTH, SCREEN_HEIGHT, HORIZONTAL_FOV, BACKGROUND_COLOR, DISTANCE_DARKNESS_COEFFICIENT,
        WALL_DEFAULT_COLOR, BLOCK_DEFAULT_COLOR, SURFACE_DEFAULT_COLOR,
    );

    let mut game = game::Game::new_game(&renderer_data).unwrap();
    let mut audio = Audio::init();
    let mut menu = Menu::new();
    
    let mut app_state = AppState::StartScreen;

    while window.is_open() {
        let escape_clicked = window.is_key_pressed(Key::Escape, KeyRepeat::No);

        match app_state {
            AppState::StartScreen => {
                window.set_cursor_visibility(true);
                
                if escape_clicked { 
                    break; 
                } else {
                    app_state = menu.update_and_draw_start_menu(&mut window, &mut buffer, &mut game, &renderer_data, &mut audio);
                }
            }

            AppState::Playing => {
                window.set_cursor_visibility(false);
                
                if escape_clicked {
                    app_state = AppState::GameExited;
                } else if game.player.hp <= 0.0 {
                    app_state = AppState::GameOver;
                } else {
                    game.update(&window, &renderer_data, &mut audio);
                    render::draw_screen(&mut buffer, &renderer_data, &game);

                    frame_count += 1;
                    let elapsed = last_time.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        fps_value = frame_count as f32 / elapsed;
                        frame_count = 0;
                        last_time = Instant::now();
                        window.set_title(&format!("DoomSquad | FPS: {:.1}", fps_value));
                    }
                }
            }

            AppState::GameExited => {
                window.set_cursor_visibility(true);
                
                if escape_clicked {
                    app_state = AppState::StartScreen;
                } else {
                    app_state = menu.update_and_draw_game_exited(&mut window, &mut buffer, &mut game, &mut audio);
                }
            }

            AppState::GameOver => {
                window.set_cursor_visibility(true);
                
                if escape_clicked {
                    app_state = AppState::StartScreen;
                } else {
                    app_state = menu.update_and_draw_game_over(&mut window, &mut buffer, &mut game);
                }
            }

            AppState::Quit => {
                break; 
            }
        }

        if let Err(e) = window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
    }
    Ok(())
}