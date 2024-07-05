use crate::position::WorldPosition;
use crate::settings::Settings;
use bevy::input::mouse::MouseMotion;
use bevy::math::f64::DVec3;
use bevy::prelude::*;
use bevy::render::view::{GpuCulling, NoCpuCulling};
use bevy::window::CursorGrabMode;

#[derive(Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct ThisPlayer;

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    _p: Player,
    world_position: WorldPosition,
}

fn init_this_player(mut commands: Commands) {
    let camera_bundle = Camera3dBundle {
        transform: Transform::from_xyz(0.0, 100., 12.0).looking_at(Vec3::new(0., 0., 0.0), Vec3::Z),
        ..default()
    };

    let mut world_position = WorldPosition::from_xyz(0.0, 100.0, 12.0);
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

fn camera_mover(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut WorldPosition, With<ThisPlayer>>,
    mut window_query: Query<&mut Window>,
) {
    // handle mouse locking
    let mut window = window_query.single_mut();
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }

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

    worldpos.position += speed * direction.normalize();
}

fn camera_rotator(
    mut camera_query: Query<(&ThisPlayer, &mut WorldPosition)>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut window_query: Query<&mut Window>,
    settings: Res<Settings>
) {
    let (_, mut worldpos) = camera_query.single_mut();
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


#[derive(Component)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_this_player)
            .add_systems(Update, (camera_mover, camera_rotator));
    }
}
