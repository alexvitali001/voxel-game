use crate::{chunk::chunk::CHUNK_SIZE_I32, player::ThisPlayer};
use bevy::{math::f64::DVec3, prelude::*};
use bevy_math::{CompassOctant, CompassQuadrant};
#[derive(Default, Debug, Component)]
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


    /*
    Very clearly and expressly documenting What The Yaw Means. Beacuse I Spent Several Hours Futizng With Trigonometry
    And Flashing What Look To The Untrained Eyes As Gang Symbols With My Right Hand And I Do Not Want To Think About
    This Ever Again:

    +------+----------+-----------+------------+
    | Deg. | self.yaw | direction | forward is |
    +------+----------+-----------+------------+
    | 0°   |        0 |     NORTH | positive X |
    +------+----------+-----------+------------+
    | 90°  |      π/2 |      EAST | positive Z |
    +------+----------+-----------+------------+
    | 180° |        π |     SOUTH | negative X |
    +------+----------+-----------+------------+
    | 270° |     3π/2 |      WEST | negative Z |
    +------+----------+-----------+------------+
    */

    // returns a vec3 corresponding to the given direction, ignoring Y coord
    pub fn forward(&self) -> DVec3 {
        return DVec3::new(self.yaw.cos().into(), 0.0, self.yaw.sin().into());
    }

    pub fn backward(&self) -> DVec3 {
        return -self.forward();
    }

    pub fn right(&self) -> DVec3 {
        let theta = self.yaw + std::f32::consts::FRAC_PI_2;
        return DVec3::new(theta.cos().into(), 0.0, theta.sin().into());
    }

    pub fn left(&self) -> DVec3 {
        return -self.right();
    }


    // get bevy compass direction 
    // These are (z, x) instead of (x, z) to make the already existing bevy compass quadrant code spit out the right compass points
    // is this confusing? we may want to just roll this ourselves?
    pub fn get_compass_quadrant(&self) -> CompassQuadrant {
        (Dir2::new(Vec2::new(self.forward().z as f32, self.forward().x as f32))).unwrap().into()
    }

    pub fn get_compass_octant(&self) -> CompassOctant {
        (Dir2::new(Vec2::new(self.forward().z as f32, self.forward().x as f32))).unwrap().into()
    }

    // set the given transform to match this WorldPosition, relative to origin
    // used in rendering to allow 64 bit floats to be used for physics
    pub fn to_render_transform(&self, origin: &Self, out: &mut Transform) {
        let x = (self.position.x - origin.position.x) as f32;
        let y = (self.position.y - origin.position.y) as f32;
        let z = (self.position.z - origin.position.z) as f32;

        out.translation = Vec3::new(x, y, z);
        // axis is -Y instead of Y to get the rotation to not be backwards
        out.rotation = Quat::from_axis_angle(Vec3::NEG_Y, self.yaw + std::f32::consts::FRAC_PI_2) * 
            Quat::from_axis_angle(Vec3::X, self.pitch);
    }

    pub fn from_xyz(x: f64, y: f64, z: f64) -> Self {
        return Self {
            position: DVec3::new(x, y, z),
            ..default()
        };
    }

    pub fn get_chunk_position(&self) -> IVec3 {
        // society when integer division rounds towards zero and not down and i have to do this
        let chunk_size = CHUNK_SIZE_I32 as f64;
        return IVec3::new(
            (self.position.x / chunk_size).floor() as i32,
            (self.position.y / chunk_size).floor() as i32,
            (self.position.z / chunk_size).floor() as i32
        )
    }

    pub fn get_within_chunk_position(&self) -> Vec3 {
        let chunk_size = CHUNK_SIZE_I32 as f64;
        
        let m = |n : f64| {
            let mut k = n % chunk_size;
            if k < 0.0 {
                k += chunk_size;
            }
            k as f32
        };

        return Vec3::new(
            m(self.position.x),
            m(self.position.y),
            m(self.position.z),
        )        
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