use bevy::window::PrimaryWindow;
use bevy::{
    asset::Assets,
    prelude::*,
    render::render_resource::{
        AddressMode::*, Extent3d, FilterMode::*, SamplerDescriptor, TextureDimension, TextureFormat,
    },
};
use block::basicblock::BlockMaterial;
use block::blockregistry::BlockRegistry;
use block::chunk::Chunk;
use block::mesh::bake;
use block_mesh::VoxelVisibility;
mod debugtext;
mod player;
mod position;

use crate::block::basicblock::BasicBlock;
use crate::debugtext::DebugTextPlugin;
use crate::player::PlayerPlugin;

use position::*;
mod block;

fn main() {
    let image_plugin = ImagePlugin {
        default_sampler: SamplerDescriptor {
            address_mode_u: Repeat,
            address_mode_v: Repeat,
            mag_filter: Nearest,
            ..default()
        },
    };
    App::new()
        .add_plugins(DefaultPlugins.set(image_plugin))
        .add_plugins((PlayerPlugin, DebugTextPlugin))
        .insert_resource(BlockRegistry::new())
        .add_systems(Startup, set_window_title)
        .add_systems(Startup, (build_block_registry, setup).chain())
        .add_systems(Update, translate_all_world_transforms)
        .run();
}

// Sets window title to proper name of game
fn set_window_title(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.title = "Diřłakū".to_string();
    }
}

fn build_block_registry(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut block_registry: ResMut<BlockRegistry>,
) {
    block_registry.register_block(
        &mut materials,
        BasicBlock {
            name: String::from("air"),
            mesh_visibility: VoxelVisibility::Empty,
            material_type: BlockMaterial::Empty,
        },
    );

    block_registry.register_block(
        &mut materials,
        BasicBlock {
            name: String::from("stone"),
            mesh_visibility: VoxelVisibility::Opaque,
            material_type: BlockMaterial::Solid(asset_server.load("textures/block/stone.png")),
        },
    );
    block_registry.register_block(
        &mut materials,
        BasicBlock {
            name: String::from("dirt"),
            mesh_visibility: VoxelVisibility::Opaque,
            material_type: BlockMaterial::Solid(asset_server.load("textures/block/dirt.png")),
        },
    );
}

// summons test shit
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    block_registry: Res<BlockRegistry>,
) {
    let debug_texture = asset_server.load("textures/block/debug.png");
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(debug_texture),
        ..default()
    });

    let test_torus = meshes.add(shape::Torus::default().into());

    commands
        .spawn((PbrBundle {
            mesh: test_torus,
            material: debug_material.clone(),
            ..default()
        },))
        .insert(WorldPosition::from_xyz(0.0, 2.0, 0.0));

    ambient_light.color = Color::ALICE_BLUE;
    ambient_light.brightness = 0.4;

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 9000.0,
                range: 1000.,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        })
        .insert(WorldPosition::from_xyz(8.0, 20.0, 8.0));

    // ground plane
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(50.0).into()),
            material: materials.add(Color::SILVER.into()),
            ..default()
        })
        .insert(WorldPosition::from_xyz(0.0, -100.0, 0.0));

    // test chunk

    println!("making chunks");

    for x in -10..=10 {
        for z in -10..=10 {
            for y in -1..=0 {
                let chunk = Chunk::generate_chunk(&block_registry, x, y, z);
                let chunk_meshes = bake(&block_registry, &chunk);
                for (bid, mesh) in chunk_meshes {
                    if let Some(mat) = block_registry.material_from_id(&bid) {
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh),
                                material: mat.clone(),
                                ..default()
                            })
                            .insert(WorldPosition::from_xyz(
                                (32 * x) as f64,
                                (32 * y) as f64,
                                (32 * z) as f64,
                            ));
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
