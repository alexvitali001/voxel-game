use crate::chunk::chunk::{CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::{player::ThisPlayer, settings::Settings};
use crate::position::universe_transform::UniverseTransform;
use crate::world::universe::Universe;
use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*, render::diagnostic::RenderDiagnosticsPlugin
};
use bevy_egui::{egui, EguiContexts};
use bevy_math::CompassOctant;

pub fn display_debug_menu(
    mut egui: EguiContexts,
    mut ds: ResMut<DebugInfo>,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<&UniverseTransform, With<ThisPlayer>>,
    universe: Res<Universe>
) {
    egui::Window::new("Debug Info").show(egui.ctx_mut(), |ui| {
        ui.checkbox(&mut ds.show_perf_info, "Show Performance Info");
        ui.checkbox(&mut ds.show_game_info, "Show Game Info");
        ui.checkbox(&mut ds.draw_chunk_borders, "Draw Chunk Borders");

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
            ui.heading("Player Info");
            let player_utrans = player_query.single();

            let dim = player_utrans.loc.dimension;
            ui.label(format!("Dimension: {}", dim));

            let pos = player_utrans.loc.position;
            ui.label(format!("Position: X {:.2}, Y {:.2}, Z {:.2}", pos.x, pos.y, pos.z));
            
            let pitch = player_utrans.pitch.to_degrees();
            let yaw = player_utrans.yaw.to_degrees();
            ui.label(format!("Rotation: Pitch {:.1}°, Yaw {:.2}°", pitch, yaw));
            
            let facing_direction = match player_utrans.get_compass_octant() {
                CompassOctant::North => "+X (North)",
                CompassOctant::NorthEast => "+X +Z (Northeast)",
                CompassOctant::East => "+Z (East)",
                CompassOctant::SouthEast => "-X +Z (Southeast)",
                CompassOctant::South => "-X (South)",
                CompassOctant::SouthWest => "-X -Z (Southwest)",
                CompassOctant::West => "-Z (West)",
                CompassOctant::NorthWest => "+X -Z (Northwest)"
            };

            ui.label(format!("Facing: {}", facing_direction));

            let fwv = player_utrans.forward();
            ui.label(format!("Forward Vector: X {:.2}, Y {:.2}, Z {:.2}", fwv.x, fwv.y, fwv.z));

            let fcv = player_utrans.facing_direction();
            ui.label(format!("Facing Vector: X {:.2}, Y {:.2}, Z {:.2}", fcv.x, fcv.y, fcv.z));


            let player_chunk = player_utrans.get_chunk_position();
            ui.label(format!("Current Chunk: X {} Y {} Z {}", player_chunk.x, player_chunk.y, player_chunk.z));

            ui.heading("Biome Info");
            let cont = universe.dimension_noise.get_splined_cont(pos.x as i32, pos.z as i32);
            ui.label(format!("Continentalness: {:.1}", cont));
        }
    });
}

fn render_chunk_borders(
    mut gizmos: Gizmos,
    settings: Res<Settings>,
    player_query: Query<&UniverseTransform, With<ThisPlayer>>
) {
    let player_utrans = player_query.single();

    let chunk_corner = -player_utrans.get_within_chunk_position();
    gizmos.grid_3d(
        chunk_corner,
        Quat::IDENTITY,
        UVec3::from_array([2 * settings.horizontal_render_distance as u32, 2 * settings.vertical_render_distance as u32, 2 * settings.horizontal_render_distance as u32]), 
        Vec3::from_array([CHUNK_SIZE as f32, CHUNK_SIZE as f32, CHUNK_SIZE as f32]),
        bevy::color::palettes::css::GREEN
    ).outer_edges();


    // the "draw 2d grid in 3d" gizmo had weirdness with the quats so
    // i just rolled my own
    let positions = vec![
        (Vec3::X, Vec3::Z, Vec3::Y), // top bottom 
        (Vec3::X, Vec3::Y, Vec3::Z), // east west
        (Vec3::Z, Vec3::Y, Vec3::X)  // north south
    ];
    let full = CHUNK_SIZE_I32 as f32;
    let spacing = 2;
    positions.iter().for_each(|(v1, v2, shift)| {
        for i in 1..=CHUNK_SIZE/spacing {
            for j in 1..=CHUNK_SIZE/spacing {
                for k in [Vec3::ZERO, *shift * full] {
                    let d1 = (((i*spacing) as f32) * *v1);
                    let d2 = (((j*spacing) as f32) * *v2);
                    let origin = chunk_corner + k;
                    gizmos.line(
                        origin + d1,
                        origin + d1 + d2,
                        bevy::color::palettes::css::RED
                    );

                    gizmos.line(
                        origin + d2,
                        origin + d2 + d1,
                        bevy::color::palettes::css::RED
                    )
                }
            }
        }
    })
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
    pub draw_chunk_borders: bool
}

const DEFAULT_DEBUG_STATE : DebugInfo = DebugInfo {
    show_all_info: true, 
    show_perf_info: true,
    show_game_info: true,
    draw_chunk_borders: false
};

pub struct DebugTextPlugin;
impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin, RenderDiagnosticsPlugin))
            .insert_resource(DEFAULT_DEBUG_STATE)
            .add_systems(Update, toggle_debug_info)
            .add_systems(Update, (
                display_debug_menu, // runs only if the master checkbox is toggled
                render_chunk_borders.run_if(|ds : Res<DebugInfo> | {ds.draw_chunk_borders})
            ).run_if(|ds : Res<DebugInfo> | {ds.show_all_info}));
    }
}
