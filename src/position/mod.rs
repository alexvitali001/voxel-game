use crate::player::ThisPlayer;
use bevy::{math::f64::DVec3, prelude::*};
#[derive(Default, Component)]
pub struct WorldPosition {
    pub position: DVec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl WorldPosition {
    // add some value to the yaw, modulo 2*pi if necessary
    pub fn add_yaw(&mut self, dyaw: f32) {
        self.yaw += dyaw;
        self.yaw %= std::f32::consts::TAU;
        if self.yaw < 0.0 {
            self.yaw += std::f32::consts::TAU;
        }
    }

    // add some value to the pitch (clamping to keep within plus/minus pi/2)
    pub fn add_pitch_clamp(&mut self, dpitch: f32) {
        let pi_halves = std::f32::consts::FRAC_PI_2;
        self.pitch = (self.pitch + dpitch).clamp(-pi_halves, pi_halves);
    }

    // returns a vec3 corresponding to the given direction, ignoring Y coord
    pub fn backward(&self) -> DVec3 {
        return DVec3::new(self.yaw.sin().into(), 0.0, self.yaw.cos().into());
    }

    pub fn forward(&self) -> DVec3 {
        return -self.backward();
    }

    pub fn right(&self) -> DVec3 {
        let theta = self.yaw + std::f32::consts::FRAC_PI_2;
        return DVec3::new(theta.sin().into(), 0.0, theta.cos().into());
    }

    pub fn left(&self) -> DVec3 {
        return -self.right();
    }

    // set the given transform to match this WorldPosition, relative to origin
    // used in rendering to allow 64 bit floats to be used for physics
    pub fn to_render_transform(&self, origin: &Self, out: &mut Transform) {
        let x = (self.position.x - origin.position.x) as f32;
        let y = (self.position.y - origin.position.y) as f32;
        let z = (self.position.z - origin.position.z) as f32;

        out.translation = Vec3::new(x, y, z);
        out.rotation = Quat::from_axis_angle(Vec3::Y, self.yaw)
            * Quat::from_axis_angle(Vec3::NEG_X, self.pitch);
    }

    pub fn from_xyz(x: f64, y: f64, z: f64) -> Self {
        return Self {
            position: DVec3::new(x, y, z),
            ..default()
        };
    }
}

// shift any entity with both a Transform and a WorldPosition to be relative to the player
pub fn translate_all_world_transforms(
    mut to_move: Query<(&mut Transform, &WorldPosition)>,
    player: Query<&WorldPosition, With<ThisPlayer>>,
) {
    let player_world_position = player.single();

    for (transform, world_position) in to_move.iter_mut() {
        world_position.to_render_transform(player_world_position, transform.into_inner());
    }
}
