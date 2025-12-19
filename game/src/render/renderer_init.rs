pub struct RendererData {
    pub screen_width_as_f64: f64,
    pub screen_height_as_f64: f64,
    pub horizontal_fov: f64,
    pub vertical_fov: f64,
    pub vertical_scale_coefficient: f64,
    pub projection_plane_distance: f64,
    pub background_color: u32,
    pub wall_default_color: u32,
    pub bottom_block_default_color: u32,
    pub top_block_default_color: u32,
    pub distance_darkness_coefficient: f64,
}

pub fn render_init(
    screen_width: usize,
    screen_height: usize,
    horizontal_fov: f64,
    background_color: u32,
    distance_darkness_coefficient: f64,
    wall_default_color: u32,
    bottom_block_default_color: u32,
    top_block_default_color: u32,
) -> RendererData {
    let screen_width_as_f64 = screen_width as f64;
    let screen_height_as_f64 = screen_height as f64;

    let vertical_fov: f64 =
        ((screen_height_as_f64 / screen_width as f64) * (horizontal_fov / 2.0).tan()).atan() * 2.0;

    let vertical_scale_coefficient: f64 = (screen_height as f64 / 2.0) / (vertical_fov / 2.0).tan();

    let projection_plane_distance: f64 = (screen_width as f64 / 2.0) / (horizontal_fov / 2.0).tan();

    RendererData {
        screen_width_as_f64,
        screen_height_as_f64,
        horizontal_fov,
        vertical_fov,
        vertical_scale_coefficient,
        projection_plane_distance,
        background_color,
        distance_darkness_coefficient,
        wall_default_color,
        bottom_block_default_color,
        top_block_default_color,
    }
}
