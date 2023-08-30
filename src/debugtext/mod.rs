use crate::player::ThisPlayer;
use crate::position::WorldPosition;
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
        .with_text_alignment(TextAlignment::Left)
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
) {
    let mut text = text_query.single_mut();

    // Position Values
    let player_worldpos = player_query.single();
    let pos = player_worldpos.position;
    let pitch = player_worldpos.pitch.to_degrees();
    let yaw = player_worldpos.yaw.to_degrees();

    // Diagnostics
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .unwrap()
        .smoothed()
        .unwrap_or_default();
    text.sections[0].value = format!(
        "X={:.5}, Y={:.5}, Z={:.5}
            \nPitch={:.1}, Yaw={:.1}
            \nFPS={:.1}",
        pos.x, pos.y, pos.z, pitch, yaw, fps
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
