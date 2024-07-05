use crate::player::ThisPlayer;
use crate::position::WorldPosition;
use crate::world::universe::Universe;
use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*, render::diagnostic::RenderDiagnosticsPlugin
};
use bevy_egui::{egui, EguiContexts};
use bevy_math::CompassOctant;

pub fn display_debug_checkbox(
    mut egui: EguiContexts,
    mut ds: ResMut<DebugInfo>,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<&WorldPosition, With<ThisPlayer>>,
    universe: Res<Universe>
) {
    egui::Window::new("Debug Info").show(egui.ctx_mut(), |ui| {
        ui.checkbox(&mut ds.show_perf_info, "Show Performance Info");
        ui.checkbox(&mut ds.show_game_info, "Show Game Info");

        if ds.show_perf_info {
            ui.heading("Performance Info");
            ui.label(format!(
                "FPS: {:.02}", diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS).unwrap().smoothed().unwrap_or_default()
            ));
    
            ui.label(format!(
                "Entities: {:.0}", diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT).unwrap().smoothed().unwrap_or_default()
            ));
        }

        if ds.show_game_info {
            ui.heading("Position Info");
            let player_worldpos = player_query.single();
            let pos = player_worldpos.position;
            ui.label(format!("Position: X {:.2}, Y {:.2}, Z {:.2}", pos.x, pos.y, pos.z));
            
            let pitch = player_worldpos.pitch.to_degrees();
            let yaw = player_worldpos.yaw.to_degrees();
            ui.label(format!("Rotation: Pitch {:.1}°, Yaw {:.2}°", pitch, yaw));
            
            let facing_direction = match player_worldpos.get_compass_octant() {
                CompassOctant::North => "+Z (North)",
                CompassOctant::NorthEast => "+X +Z (Northeast)",
                CompassOctant::East => "+X (East)",
                CompassOctant::SouthEast => "+X -Z (Southeast)",
                CompassOctant::South => "-Z (South)",
                CompassOctant::SouthWest => "-X -Z (Southwest)",
                CompassOctant::West => "-X (West)",
                CompassOctant::NorthWest => "-X +Z (Northwest)"
            };

            ui.label(format!("Facing: {}", facing_direction));

            let forward = player_worldpos.forward();
            ui.label(format!("Forward: {:.2} {:.2} {:.2}", forward.x, forward.y, forward.z));

            let player_chunk = player_worldpos.get_chunk_position();
            ui.label(format!("Current Chunk: X {} Y {} Z {}", player_chunk.x, player_chunk.y, player_chunk.z));

            ui.heading("Biome Info");
            let cont = universe.dimension_noise.get_splined_cont(pos.x as i32, pos.z as i32);
            ui.label(format!("Continentalness: {:.1}", cont));
        }
    });
}

pub fn toggle_debug_info(keys: Res<ButtonInput<KeyCode>>, mut ui_state: ResMut<DebugInfo>) {
    for key in keys.get_just_pressed() {
        if key == &KeyCode::F3 {
            ui_state.show_all_info = !ui_state.show_all_info;
        }
    }
}

#[derive(Resource)]
pub struct DebugInfo {
    pub show_all_info : bool,
    pub show_perf_info : bool,
    pub show_game_info : bool,
}

const DEFAULT_DEBUG_STATE : DebugInfo = DebugInfo {
    show_all_info: true, 
    show_perf_info: true,
    show_game_info: true,
};

pub struct DebugTextPlugin;
impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin, RenderDiagnosticsPlugin))
            .insert_resource(DEFAULT_DEBUG_STATE)
            .add_systems(Update, toggle_debug_info)
            .add_systems(Update, (
                display_debug_checkbox, // runs only if the master checkbox is toggled
            ).run_if(|ds : Res<DebugInfo> | {ds.show_all_info}));
    }
}
