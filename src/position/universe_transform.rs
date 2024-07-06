use bevy::{math::f64::DVec3, prelude::*};
use bevy_math::{CompassOctant, CompassQuadrant, DQuat};
use super::universe_location::*;

#[derive(Default, Debug, Component, Clone)]
pub struct UniverseTransform {
    pub loc: UniverseLocation,
    pub pitch: f64,
    pub yaw: f64,
}

impl UniverseTransform {
    // add some value to the yaw, modulo 2*pi if necessary
    pub fn add_yaw(&mut self, dyaw: f64) {
        self.yaw += dyaw;
        self.yaw %= std::f64::consts::TAU;
        if self.yaw < 0.0 {
            self.yaw += std::f64::consts::TAU;
        }
    }

    // add some value to the pitch (clamping to keep within plus/minus pi/2)
    pub fn add_pitch_clamp(&mut self, dpitch: f64) {
        let pi_halves = std::f64::consts::FRAC_PI_2;
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
        return DVec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
    }

    pub fn backward(&self) -> DVec3 {
        return -self.forward();
    }

    pub fn right(&self) -> DVec3 {
        let theta = self.yaw + std::f64::consts::FRAC_PI_2;
        return DVec3::new(theta.cos(), 0.0, theta.sin());
    }

    pub fn left(&self) -> DVec3 {
        return -self.right();
    }


    // gets the quaternion that corresponds to this UniverseTransform's rotation
    // required a lot of stupid trial and error and magic vectors.
    fn rotating_quaternion(&self) -> DQuat {
        DQuat::from_axis_angle(DVec3::NEG_Y, self.yaw as f64 + std::f64::consts::FRAC_PI_2) * 
        DQuat::from_axis_angle(DVec3::X, self.pitch as f64)
    }

    // returns the vector pointing in the direction that this UniverseTransform is facing
    pub fn facing_direction(&self) -> DVec3 {
        self.rotating_quaternion().mul_vec3(DVec3::NEG_Z)
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
        let x = (self.loc.position.x - origin.loc.position.x) as f32;
        let y = (self.loc.position.y - origin.loc.position.y) as f32;
        let z = (self.loc.position.z - origin.loc.position.z) as f32;

        out.translation = Vec3::new(x, y, z);
        // axis is -Y instead of Y to get the rotation to not be backwards
        out.rotation = self.rotating_quaternion().as_quat(); // cast to f32 quat
    }

    pub fn from_dim_xyz<T>(dimension: u32, position: T) -> Self
        where T: Into<DVec3> {
        return Self {
            loc: UniverseLocation::from_dim_xyz(dimension, position),
            ..default()
        };
    }

    pub fn get_chunk_position(&self) -> IVec3 {
        self.loc.get_chunk_position()
    }

    pub fn get_within_chunk_position(&self) -> Vec3 {
        self.loc.get_within_chunk_position()      
    }
}