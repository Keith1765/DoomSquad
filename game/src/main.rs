#![allow(dead_code)]

mod game;
mod render;

use minifb::{Key, MouseMode, Window, WindowOptions};

use std::time::{Duration, Instant};

const WIDTH: usize = 800;
const HEIGHT: usize = 500;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //for fps count
    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_value = 0.0;

    //creates window Safely
    let mut window = match Window::new("game", WIDTH, HEIGHT, WindowOptions::default()) {
        Ok(w) => w,
        Err(e) => {
            eprint!("failed to create Window");
            return Err(Box::new(e));
        }
    };
    window.set_cursor_visibility(false); // hide mouse 

    //to reduce CPU load by decreasing refresh rate oder so lol
    window.set_target_fps(60);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut game = game::Game::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        game.update(&window);

        render::draw(&mut buffer, &mut game);

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
        if let Err(e) = window.update_with_buffer(&buffer, WIDTH, HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
    }
    Ok(())
}
