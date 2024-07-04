use crate::player::ThisPlayer;
use crate::position::WorldPosition;
use crate::world::universe::Universe;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
#[derive(Component)]
struct DebugText;

fn setup_debug_text(mut commands: Commands) {
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            "Position: ",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ) // Set the alignment of the Text
        .with_text_justify(JustifyText::Left)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        DebugText,
    ));
}

fn update_debug_text(
    mut text_query: Query<&mut Text, With<DebugText>>,
    player_query: Query<&WorldPosition, With<ThisPlayer>>,
    diagnostics: Res<DiagnosticsStore>,
    universe: Res<Universe>
) {
    let mut text = text_query.single_mut();

    // Position Values
    let player_worldpos = player_query.single();
    let pos = player_worldpos.position;
    let pitch = player_worldpos.pitch.to_degrees();
    let yaw = player_worldpos.yaw.to_degrees();
    let player_chunk = player_worldpos.get_chunk_position();

    // Diagnostics
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .unwrap()
        .smoothed()
        .unwrap_or_default();
    text.sections[0].value = format!(
        "XYZ={:.5}, {:.5}, {:.5}
            \nChunk={}, {}, {}
            \nPitch={:.1}, Yaw={:.1}
            \nFPS={:.1}
            \nContinentalness={:.1}",
        pos.x, pos.y, pos.z, 
        player_chunk.x, player_chunk.y, player_chunk.z,
        pitch, yaw, 
        fps, 
        universe.dimension_noise.get_splined_cont(pos.x as i32, pos.z as i32),
    );
}

pub struct DebugTextPlugin;
impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_debug_text)
            .add_systems(Update, update_debug_text);
    }
}
