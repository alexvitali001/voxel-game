use bevy::prelude::*;

#[derive(Resource)]
pub struct Settings {
    // rendering
    pub horizontal_render_distance: u8, // do i expect anyone to break the bounds of a u8? no. should i give the foolish the option? maybe later.
    pub vertical_render_distance: u8,

    // controls
    pub mouse_sensitivity: f32
}

pub const DEFAULT_SETTINGS : Settings = Settings {
    horizontal_render_distance: 8,
    vertical_render_distance: 8,

    mouse_sensitivity: 0.005
};

