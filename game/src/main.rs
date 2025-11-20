#![allow(dead_code)]

use minifb::{Key, Window, WindowOptions};

use std::{f32::consts::PI};

#[derive(Clone, Copy)]
struct Player {
    px:f32,
    py:f32,
    pdx:f32,
    pdy:f32,
    pa:f32,
}


const WIDTH: usize = 800;
const HEIGHT: usize = 560;
const MOVESPEED: f32 = 2.0;
const ROTATIONSPEED: f32 = 3.0;
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
        for x in 0..MAPX {
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
                        if py == yo || py == yo+MAPSIZE||px == xo ||px == xo + MAPSIZE{
                           buffer[py*WIDTH+px] = 0xFF00FF00; 
                        } else {
                            buffer[py*WIDTH+px] = color;
                        }
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

fn draw_line(buffer: &mut [u32], x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    // Convert to signed for math (avoids underflow)
    let mut x0 = x0 as isize;
    let mut y0 = y0 as isize;
    let x1 = x1 as isize;
    let y1 = y1 as isize;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // Only draw inside the screen
        if x0 >= 0 && x0 < WIDTH as isize && y0 >= 0 && y0 < HEIGHT as isize {
            buffer[y0 as usize * WIDTH + x0 as usize] = color;
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}


fn move_player (window: &Window, player: & mut Player) {
    
    if window.is_key_down(Key::A) {
        if player.pa <0.1 {player.pa += 2.0*PI}
        player.pa -= 0.1;
        player.pdx = player.pa.cos()*ROTATIONSPEED;
        player.pdy = player.pa.sin()*ROTATIONSPEED;
    }
    if window.is_key_down(Key::D) {
        if player.pa > 2.0*PI {player.pa -= 2.0*PI}
        player.pa += 0.1;
        player.pdx = player.pa.cos()*ROTATIONSPEED;
        player.pdy = player.pa.sin()*ROTATIONSPEED; 
    }
    if window.is_key_down(Key::W) {
        player.px += player.pdx;
        player.py += player.pdy;
    }
    if window.is_key_down(Key::S) {
        player.px -= player.pdx;
        player.py -= player.pdy;
    }
}


fn draw_player (buffer: &mut [u32], player: Player) {
    //make player thicccker but have to check for out of bounds
    let x = player.px as isize;
    let y = player.py as isize;

    for dx in -1..=1 {
        for dy in -1..=1 {
            let px = x + dx;
            let py = y + dy;

            if px >= 0 && px < WIDTH as isize && py >= 0 && py < HEIGHT as isize {
                let ux = px as usize;
                let uy = py as usize;
                let index = uy * WIDTH + ux;
                buffer[index] = 0xFFFF0000;
            }
        }
    }
    let x1f = player.px+player.pdx*5.0;
    let y1f = player.py+player.pdy*5.0;

    let x1 = x1f.clamp(0.0, (WIDTH - 1) as f32) as usize;
    let y1 = y1f.clamp(0.0, (HEIGHT - 1) as f32) as usize;
    let x0 = player.px.clamp(0.0, (WIDTH-1) as f32) as usize;
    let y0 = player.py.clamp(0.0, (WIDTH-1) as f32) as usize;

    draw_line(buffer, x0, y0, x1, y1, 0xFFFF0000);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //initialize player
    let pa:f32=-PI/2.0;
    let mut player =Player {
        px : 300.0,
        py : 300.0,
        pdx : pa.cos()*ROTATIONSPEED,
        pdy : pa.sin()*ROTATIONSPEED,
        pa ,
    };

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
