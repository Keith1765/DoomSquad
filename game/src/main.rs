#![allow(dead_code)]

use minifb::{Key, Window, WindowOptions};

type Player = (usize,usize);

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const STEP: usize = 3;
const MAPX: usize = 8;
const MAPY: usize = 8;
const MAPSIZE: usize = 64;
const MAP: [usize;64] = [
    1,1,1,1,1,1,1,1,
    1,0,1,0,0,0,0,1,
    1,0,1,0,0,0,0,1,
    1,0,1,0,0,0,0,1,
    1,0,1,0,0,1,0,1,
    1,0,0,0,0,1,0,1,
    1,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1
];

fn draw_map (buffer: &mut [u32]) {
    for y in 0..MAPY {
        for x in 0..MAPY {
            let color: u32 = if MAP[y * MAPX + x] == 1 {
                0xFF000000 // The Dark Side muhahaha
            } else {
                0xFFFFFFFF //white
            };

            //Top left corner of current cell
            let xo = x*MAPSIZE;
            let yo = y*MAPSIZE;

            for py in yo..yo+MAPSIZE{
                for px in xo..xo+MAPSIZE{
                    if px < WIDTH && py < HEIGHT {
                        buffer[py*WIDTH+px] = color;
                    }
                }
            }


        }
    }
}

fn put_pixel (buffer: &mut [u32], x: usize, y: usize, color: u32){
    buffer[y* WIDTH+x] = color;
}

fn draw_vertical_line (buffer: &mut [u32], x: usize, start: usize, end: usize, color: u32){
    for y in start..end {
        buffer[y * WIDTH + x] = color;
    }
}

fn move_player (window: &Window, player: & mut Player) {
    
    if window.is_key_down(Key::W) {player.1 = player.1.saturating_sub(STEP)}
    if window.is_key_down(Key::A) {player.0 = player.0.saturating_sub(STEP)}
    if window.is_key_down(Key::S) {player.1 = (player.1+STEP).min(HEIGHT-1)}
    if window.is_key_down(Key::D) {player.0 = (player.0+STEP).min(WIDTH-1)}
}


fn draw_player (buffer: &mut [u32], player: Player) {
    //make player thicccker but have to check for out of bounds
    for x in player.0.saturating_sub(1)..player.0+1.min(HEIGHT-1) {
        for y in player.1.saturating_sub(1)..player.1+1.min(WIDTH-1) {
            
            put_pixel(buffer, x , y, 0xFF0000);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //initialize player
    let mut player: Player = (300,300);

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

    while window.is_open() && !window.is_key_down(Key::Escape) {

        //draw shit
        //dont question this, I was playing around
        //& 0xFF converts to color
        //| | | merges colors
        for y in 0..HEIGHT{
            for x in 0..WIDTH{
                let r = (y as u32)/2 & 0xFF;
                let g = (x as u32)/3 & 0xFF;
                let b = ((y+x) as u32)/5 & 0xFF;

                buffer[y * WIDTH + x] = (r << 16) | (g<<8) | b;
            }
        }

        draw_map(& mut buffer);
        
        draw_player(&mut buffer, player);
        move_player(&window, &mut player);
       

        //show buffer safely
        if let Err(e) = window.update_with_buffer(&buffer, WIDTH, HEIGHT) {
            eprintln!("failed to update the window: {e}");
            return Err(Box::new(e));
        }
        
        }

    Ok(())
}
