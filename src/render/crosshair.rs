use crate::{SCREEN_WIDTH, game::map::Point, render::RendererData};

const CROSSHAIR_THICKNESS: usize = 2;
const CROSSHAIR_SIZE: isize = 20;

pub fn draw_crosshair(
    buffer: &mut [u32],
    vertical_aim_offset: f64,
    renderer_data: &RendererData,
    aim_mode: bool,
) {
    let crosshair_color = match aim_mode {
        true => 0x10e8e8,
        false => 0xFFFFFF,
    };

    let crosshair_vertical_offset = aim_offset_conversion(vertical_aim_offset, renderer_data);

    let middle_point: Point = Point {
        x: renderer_data.screen_width_as_f64 * 0.5,
        y: renderer_data.screen_height_as_f64 * 0.5,
    };
    //draw horizontal line
    for x in -CROSSHAIR_SIZE / 2..=CROSSHAIR_SIZE / 2 {
        for y in 0..CROSSHAIR_THICKNESS {
            buffer[((middle_point.y + crosshair_vertical_offset) as usize + y) * SCREEN_WIDTH
                + (middle_point.x as isize + x) as usize] = crosshair_color;
        }
    }

    //draw vertical line
    for y in -CROSSHAIR_SIZE / 2..=CROSSHAIR_SIZE / 2 {
        for x in 0..CROSSHAIR_THICKNESS {
            buffer[((middle_point.y + crosshair_vertical_offset) as isize + y) as usize
                * SCREEN_WIDTH
                + (middle_point.x as usize + x)] = crosshair_color;
        }
    }
}

fn aim_offset_conversion(vertical_aim_offset: f64, renderer_data: &RendererData) -> f64 {
    // let height_middle = renderer_data.screen_height_as_f64 * 0.5;
    let height_offset = (renderer_data.screen_height_as_f64 * 0.25) * vertical_aim_offset;
    height_offset.clamp(
        -renderer_data.screen_height_as_f64 * 0.5,
        renderer_data.screen_height_as_f64 * 0.5,
    ) //clamp unneccesary if math correct
}
