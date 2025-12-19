#![allow(dead_code)]

mod game;
mod render;

use crate::render::{RendererData, render_init};
use std::f64::consts::PI;
use std::time::{Duration, Instant};
use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::{event_loop::EventLoop,window::{Window, WindowAttributes}};


const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;
const TARGET_FPS: usize = 60;
const HORIZONTAL_FOV: f64 = PI / 2.0;
const BACKGROUND_COLOR: u32 = 0x222222;
const DISTANCE_DARKNESS_COEFFICIENT: f64 = 0.025;
const WALL_DEFAULT_COLOR: u32 = 0x00ff00;
const BLOCK_DEFAULT_COLOR: u32 = 0x0000ff;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // //for fps count
    // let mut last_time = Instant::now();
    // let mut frame_count = 0;
    // let mut fps_value = 0.0;

    // //creates window Safely
    // let mut window = match Window::new(
    //     "game",
    //     SCREEN_WIDTH,
    //     SCREEN_HEIGHT,
    //     WindowOptions::default(),
    // ) {
    //     Ok(w) => w,
    //     Err(e) => {
    //         eprint!("failed to create Window");
    //         return Err(Box::new(e));
    //     }
    // };
    // window.set_cursor_visibility(false); // hide mouse 

    // //to reduce CPU load by decreasing refresh rate oder so lol
    // window.set_target_fps(TARGET_FPS);

    // let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];


 let renderer_data: RendererData = render_init(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        HORIZONTAL_FOV,
        BACKGROUND_COLOR,
        DISTANCE_DARKNESS_COEFFICIENT,
        WALL_DEFAULT_COLOR,
        BLOCK_DEFAULT_COLOR,
    );

   

    //TODO make safe when works
    let event_loop = EventLoop::new().unwrap();

    let window = event_loop.create_window(
    WindowAttributes::default().with_title("DoomSquad")).unwrap();

    // Create a SurfaceTexture for pixels
    let surface = SurfaceTexture::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, &window);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface).unwrap();



event_loop.run(move |event, elwt| {
    elwt.set_control_flow(ControlFlow::Poll);

    match event {
        Event::WindowEvent { event, .. } => {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                _ => {}
            }
        }


        _ => {}
    }
});


    //let mut game = game::Game::new();

   



    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     game.update(&window);

    //     render::draw(&mut buffer, &renderer_data, &mut game);

    //     //fps calc
    //     frame_count += 1;
    //     let elapsed = last_time.elapsed().as_secs_f32();

    //     if elapsed >= 1.0 {
    //         fps_value = frame_count as f32 / elapsed;
    //         frame_count = 0;
    //         last_time = Instant::now();

    //         window.set_title(&format!("My Window | FPS: {:.1}", fps_value));
    //     }
    //     //show buffer safely
    //     if let Err(e) = window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT) {
    //         eprintln!("failed to update the window: {e}");
    //         return Err(Box::new(e));
    //     }
    // }
    Ok(())
}
