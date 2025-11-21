#![allow(dead_code)]

mod game;
mod render;

use minifb::{Key, Window, WindowOptions};


const WIDTH: usize = 800;
const HEIGHT: usize = 560;





fn main() -> Result<(), Box<dyn std::error::Error>> {

    //creates window Safely
    let mut window = match Window::new(
        "game", 
        WIDTH, 
        HEIGHT, 
        WindowOptions::default()
    ) {
        Ok(w) => w,
        Err(e) => {
            eprint!("failed to create Window");
            return Err(Box::new(e));
        }
    };

    //to reduce CPU load by decreasing refresh rate oder so lol
    window.set_target_fps(60);
    
    let mut buffer: Vec<u32> = vec![0;WIDTH*HEIGHT];

    let mut game = game::Game::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {


        game.update(&window);

        render::draw(&mut buffer, &game);
       

        //show buffer safely
        if let Err(e) = window.update_with_buffer(&buffer, WIDTH, HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
        
        }

    Ok(())
}
