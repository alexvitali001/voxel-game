mod debugtext;
mod player;
mod position;
mod world;
mod block;
mod registryresource;

use bevy::window::PrimaryWindow;
use bevy::{
    asset::Assets,
    prelude::*,
    render::{
        texture::{
            ImageSamplerDescriptor,
            ImageAddressMode::*,
            ImageFilterMode::*
        },
        render_resource::{
            Extent3d, TextureDimension, TextureFormat
        },
        render_asset::RenderAssetUsages,
    },
    color::palettes::basic::SILVER,
    color::palettes::css::ALICE_BLUE
};
use block::basicblock::BlockMaterial;
use block::blockregistry::BlockRegistry;
use block_mesh::VoxelVisibility;
use world::chunkmap::ChunkMap;
use registryresource::RegistryResource;
use world::gen::{ChunkEventsPlugin, GenerateChunkEvent};

use crate::block::basicblock::BasicBlock;
use crate::debugtext::DebugTextPlugin;
use crate::player::PlayerPlugin;

use position::*;

fn main() {
    let image_plugin = ImagePlugin {
        default_sampler: ImageSamplerDescriptor {
            address_mode_u: Repeat,
            address_mode_v: Repeat,
            mag_filter: Nearest,
            ..default()
        },
    };

    App::new()
        .add_plugins(DefaultPlugins.set(image_plugin))
        .add_plugins((PlayerPlugin, DebugTextPlugin, ChunkEventsPlugin))
        .insert_resource(RegistryResource::new(BlockRegistry::new()))
        .insert_resource(ChunkMap::new())
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
    block_registry_resource: ResMut<RegistryResource<BlockRegistry>>,
) {
    let br = block_registry_resource.clone_registry();
    let mut block_registry = br.write();
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
    mut ev_gen : EventWriter<GenerateChunkEvent>
) {
    let debug_texture = asset_server.load("textures/block/debug.png");
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(debug_texture),
        ..default()
    });

    let test_torus = meshes.add(Torus::default());

    commands
        .spawn((PbrBundle {
            mesh: test_torus,
            material: debug_material.clone(),
            ..default()
        },))
        .insert(WorldPosition::from_xyz(0.0, 40.0, 0.0));

    ambient_light.color = ALICE_BLUE.into();
    ambient_light.brightness = 640.0;

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 14400000.0,
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
            mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
            material: materials.add(Color::Srgba(SILVER)),
            ..default()
        })
        .insert(WorldPosition::from_xyz(0.0, -100.0, 0.0));

    // test chunk

    println!("making chunks");
    for x in -10..=10 {
        for z in -10..=10 {
            for y in -1..=0 {
                ev_gen.send(GenerateChunkEvent(IVec3::new(x,y,z)));
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
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    )
}
