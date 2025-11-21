use crate::{HEIGHT, WIDTH};
use crate::game::{Game, map::{MAPSIZE, MAPX, MAPY}};

pub fn draw(buffer: &mut [u32], game: &Game) {
    //Clear the buffer
    for px in buffer.iter_mut() {
        *px = 0x222222;
    }
    draw_map(buffer, game);
    draw_player(buffer, game);
}


fn draw_map (buffer: &mut [u32], game: &Game) {
    for y in 0..MAPY {
        for x in 0..MAPX {
            let color: u32 = if game.map.data[y * MAPX + x] == 1 {
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


fn draw_player (buffer: &mut [u32], game: &Game) {
    //make player thicccker but have to check for out of bounds
    let x = game.player.px as isize;
    let y = game.player.py as isize;

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
    let x1f = game.player.px+game.player.pdx*5.0;
    let y1f = game.player.py+game.player.pdy*5.0;

    let x1 = x1f.clamp(0.0, (WIDTH - 1) as f64) as usize;
    let y1 = y1f.clamp(0.0, (HEIGHT - 1) as f64) as usize;
    let x0 = game.player.px.clamp(0.0, (WIDTH-1) as f64) as usize;
    let y0 = game.player.py.clamp(0.0, (HEIGHT-1) as f64) as usize;

    draw_line(buffer, x0, y0, x1, y1, 0xFFFF0000);
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


