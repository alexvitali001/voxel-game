use crate::chunk::chunk::{BlockId, Chunk, AIR};
use crate::settings::Settings;
use crate::universe_transform::UniverseTransform;
use crate::world::loading::ChunkRemeshEvent;
use crate::world::universe::Universe;
use bevy::input::mouse::MouseMotion;
use bevy::math::f64::DVec3;
use bevy::prelude::*;
use bevy::render::view::{GpuCulling, NoCpuCulling};
use bevy::window::CursorGrabMode;
use zerocopy::FromBytes;

#[derive(Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct ThisPlayer;

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    _p: Player,
    world_position: UniverseTransform,
}

fn init_this_player(mut commands: Commands) {
    let camera_bundle = Camera3dBundle {
        transform: Transform::from_xyz(0.0, 100., 12.0).looking_at(Vec3::new(0., 0., 0.0), Vec3::Z),
        ..default()
    };

    let mut world_position = UniverseTransform::from_dim_xyz(0, (0.0, 100.0, 12.0));
    world_position.pitch = 1.57;
    let player_bundle = PlayerBundle {
        _p: Player,
        world_position: world_position,
    };
    commands.spawn((camera_bundle, player_bundle, ThisPlayer,
            // Enable GPU frustum culling (does not automatically disable CPU frustum culling).
            GpuCulling,
            // Disable CPU frustum culling.
            NoCpuCulling
    ));
}

fn mouse_lock_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut window_query: Query<&mut Window>,
) {
    // handle mouse locking
    // todo: force mouse to middle of screen when in locked mode,
    // be nice with the eguis
    let mut window = window_query.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn camera_mover(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut UniverseTransform, With<ThisPlayer>>,
) {
    // handle direction
    let mut worldpos = query.single_mut();
    let mut direction = DVec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += worldpos.forward();
    }
    if keys.pressed(KeyCode::KeyS) {
        direction += worldpos.backward();
    }
    if keys.pressed(KeyCode::KeyA) {
        direction += worldpos.left();
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += worldpos.right();
    }
    if keys.pressed(KeyCode::Space) {
        direction += DVec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        direction -= DVec3::Y;
    }

    if direction == DVec3::ZERO {
        return;
    }

    let speed = if keys.pressed(KeyCode::ControlLeft) {
        0.25
    } else {
        0.1
    };

    *worldpos += speed * direction.normalize();
}

fn camera_rotator(
    mut camera_query: Query<&mut UniverseTransform, With<ThisPlayer>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut window_query: Query<&mut Window>,
    settings: Res<Settings>
) {
    let mut worldpos = camera_query.single_mut();
    let window = window_query.single_mut();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let mouse_vec: Vec2 = mouse_motion_event_reader.read().map(|x| x.delta).sum();

    if mouse_vec == Vec2::ZERO {
        return;
    }

    worldpos.add_pitch_clamp((-mouse_vec.y * settings.mouse_sensitivity) as f64);
    worldpos.add_yaw((mouse_vec.x * settings.mouse_sensitivity) as f64);
}


pub const PLAYER_REACH : f64 = 5.0;
fn block_handler(
    mouse: Res<ButtonInput<MouseButton>>,
    player: Query<&UniverseTransform, With<ThisPlayer>>,
    universe: Res<Universe>,
    mut ev_remesh: EventWriter<ChunkRemeshEvent>,
    mut gizmos: Gizmos
) {
    let worldpos = player.single();
    
    // i hate the way this code looks.
    let (target_block, adjacent_block, target_block_id) = {
        let mut current = None; 
        let mut prev = None;
        let mut id = None;

        for position in worldpos.integer_raycast(PLAYER_REACH) {
            prev = current;
            current = Some(position);
            match universe.block_at(position) {
                None => { 
                    current = None;
                    prev = None;
                    id = None;  
                    break;
                } 
                Some(AIR) => {} // hit an air block, keep going
                Some(e) => { 
                    id = Some(e);
                    break;
                }
            }
        }

        match id {
            None => {(None, None, None)}
            Some(_) => {(current, prev, id)}
        }
    };

    if let Some(p) = target_block {
        gizmos.cuboid(
            Transform::from_translation((p.position - worldpos.loc.position + (0.5 * DVec3::ONE)).as_vec3()), 
            bevy::color::palettes::css::PURPLE
        );

        if mouse.just_pressed(MouseButton::Left) {
            info!("Broke block at {} {} {} (ID {})", p.position.x, p.position.y, p.position.z, target_block_id.unwrap().0);

            let chunk_pos = &p.get_chunk_position();
            let c = universe.fetch_chunk_exists(chunk_pos);
            let mut chunk = Chunk::read_from(c.as_ref()).unwrap();

            let [x, y, z] = p.get_within_chunk_position().floor().to_array();
            chunk.place(BlockId(0), (x as u32, y as u32, z as u32));
            universe.flush_chunk(chunk_pos, &chunk);
            ev_remesh.send(ChunkRemeshEvent(*chunk_pos));
        }
    }

    if let Some(ap) = adjacent_block {
        if mouse.just_pressed(MouseButton::Right) {
            info!("Placed block at {} {} {}", ap.position.x, ap.position.y, ap.position.z);
            
            let chunk_pos = &ap.get_chunk_position();
            let c = universe.fetch_chunk_exists(&ap.get_chunk_position());
            let mut chunk = Chunk::read_from(c.as_ref()).unwrap();

            let [x, y, z] = ap.get_within_chunk_position().floor().to_array();
            chunk.place(BlockId(1), (x as u32, y as u32, z as u32));
            universe.flush_chunk(chunk_pos, &chunk);
            ev_remesh.send(ChunkRemeshEvent(*chunk_pos));
        }
    }



}

// shift any entity with both a Transform and a UniverseTransform to be relative to the player
// TODO move this into the positions module
pub fn translate_all_world_transforms(
    mut to_move: Query<(&mut Transform, &UniverseTransform)>,
    player: Query<&UniverseTransform, With<ThisPlayer>>,
) {
    let player_world_position = player.single();

    for (transform, world_position) in to_move.iter_mut() {
        world_position.to_render_transform(player_world_position, transform.into_inner());
    }
}

pub fn spawn_reticle(
    asset_server: Res<AssetServer>,
    mut commands : Commands
) {
    commands.spawn(
        ImageBundle {
        image: UiImage::new(asset_server.load("textures/ui/reticle.png")),
        style: Style {
            position_type: PositionType::Relative,
            align_self: AlignSelf::Center, 
            justify_self: JustifySelf::Center,
            ..default()
        },
        ..default()}
    );
}

#[derive(Component)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_this_player)
            .add_systems(Startup, spawn_reticle)
            .add_systems(Update, (mouse_lock_handler, camera_mover, camera_rotator))
            .add_systems(Update, block_handler)
            .add_systems(PostUpdate, translate_all_world_transforms);
    }
}
