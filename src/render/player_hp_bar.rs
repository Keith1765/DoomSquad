use crate::{SCREEN_WIDTH, render::RendererData};

pub fn draw_player_hp_bar(buffer: &mut [u32], render_data: &RendererData, hp: f64) {
    // 2x multipllicator to make hp bar bigger
    let clamped_hp = (hp * 2.0).clamp(0.0, (render_data.screen_width_as_f64) - 100.0) as usize;

    for hp_bar_height in 0..5 {
        for hp_bar_length in 0..clamped_hp {
            buffer[(hp_bar_height + 20) * SCREEN_WIDTH + (hp_bar_length + 50)] = 0xFF0000;
        }
    }
}
