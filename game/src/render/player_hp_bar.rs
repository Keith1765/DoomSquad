use crate::{SCREEN_WIDTH, render::RendererData};


pub fn draw_player_hp_bar (buffer: &mut [u32], render_data: & RendererData, hp: f64) {
    let clamped_hp = hp.clamp(50.0, render_data.screen_width_as_f64) as usize;

    for hp_bar_height in 0..5 {
        for hp_bat_length in 0..clamped_hp*2{
            buffer[(hp_bar_height + 20)*SCREEN_WIDTH + (hp_bat_length + 50)] = 0xFF0000;
        }
    }

}