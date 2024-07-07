use bevy::{math::f64::DVec3, prelude::*};
use bevy_math::{CompassOctant, CompassQuadrant, DQuat};
use crate::world::universe::Universe;

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

    // gets all integer coordinates directly looked at by this worldtransform, in order, up to max_range 
    // TODO: once rust '24 drops, make this return an iterator instead of adding things to hits
    pub fn integer_raycast(&self, max_range: f64) -> Vec<UniverseLocation> {
        let mut hits = vec![];
        let max_range = if max_range < 0.0 {0.0} else {max_range}; // floats aren't Ord so no std::cmp::max

        let start = self.loc.position;
        let slope = self.facing_direction();
        let signs = slope.signum();

        
        // given a point, and an axis, move it along the slope vector
        // until it has an integer coordinate for that axis
        let next_integer_point = 
            |point: DVec3, axis: usize| {
            const EPSILON: f64 = 1e-10;
            if slope[axis].abs() < EPSILON {
                std::f64::INFINITY * signs // too far
            } else {
                // find the target integer 
                let target = {
                    let x = point[axis];
                    if (x - x.round()).abs() < EPSILON {
                        x.round() + signs[axis]
                    } else if signs[axis] > 0.0 {
                        x.ceil()
                    } else {
                        x.floor()
                    }
                };
                
                // compute the distance needed to reach the target
                let distance_to_collision = (target - point[axis]) / slope[axis];

                // move our original point that distance in regular space
                let mut new_point = point + slope * distance_to_collision;
                // attend to float imprecision and set the one axis to the mathematically guaranteed answer
                new_point[axis] = target;
                new_point
            }};

        // we always see the block we are currently in
        hits.push(self.loc.position_in_same_dimension(start.floor()));

        // our current position
        let mut cursor = start; 
        let mut num_hits = 1;

        // sanity check: if we ever have this many hits, something has gone horribly wrong.
        // the range is 3x the maximum to give debugging some more data 
        let too_many_hits = (max_range * 3.0).ceil() as i32;

        loop {
            // get candidate points
            // these are the closest points whose x/y/z
            // coords are integers
            let candidates = [
                next_integer_point(cursor, 0),
                next_integer_point(cursor, 1),
                next_integer_point(cursor, 2)
            ];

            // find the closest candidate point and the axis 
            // we traveled to get to it
            let (closest_axis, closest_candidate) = {
                let mut min_distance = std::f64::INFINITY;
                let mut min_index = 0;

                for i in [0,1,2] {
                    // distance_squared for perf
                    let d = cursor.distance_squared(candidates[i]);
                    if d < min_distance {
                        min_distance = d;
                        min_index = i;
                    }

                }

                (min_index, candidates[min_index])
            };

            cursor = closest_candidate;
            
            if start.distance(cursor) > max_range {
                break; // if this point is too far away, we can't reach the block, so stop
            } else { // otherwise, yield the appropriate block position and move on
                let mut entered_block = cursor.floor();
                // if we got here by moving backwards, we need to drop a block
                if signs[closest_axis] < 0.0 {entered_block[closest_axis] -= 1.0};
                hits.push(self.loc.position_in_same_dimension(entered_block));
            }

            num_hits += 1;
            if num_hits > too_many_hits {
                warn!("Integer raycast found too many blocks. Breaking out of potential infinite loop. This shouldn't happen.");
                break;
            }

        }
        hits

        /* 
        // previous implementation

        let nx = if signs.x > 0.0 {start.position.x.ceil()} else {start.position.x.floor()};
        let ny = if signs.y > 0.0 {start.position.y.ceil()} else {start.position.y.floor()};
        let nz = if signs.z > 0.0 {start.position.z.ceil()} else {start.position.z.floor()};

        let mut int_points = DVec3::new(nx, ny, nz);

        // nearest point with integer coordinate 
        let epsilon = 1e-7; 
        let inf_signs = std::f64::INFINITY * signs;
        let px = if slope.x.abs() < epsilon {inf_signs} else {(nx - start.position.x) * slope / slope.x + start.position};
        let py = if slope.y.abs() < epsilon {inf_signs} else {(ny - start.position.y) * slope / slope.y + start.position};
        let pz = if slope.z.abs() < epsilon {inf_signs} else {(nz - start.position.z) * slope / slope.z + start.position};

        let mut nearest_points = [px, py, pz];

        loop {
            let dists = DVec3::from_array(nearest_points.map(|p| signs.x * (p.x - start.position.x)));

            // which axis' plane intersects the line first?
            let nearest_ind = {
                let min_elem = dists.min_element();
                if dists.x == min_elem {0}
                else if dists.y == min_elem {1}
                else {2}
            };

            let mut nearest_point = nearest_points[nearest_ind].floor();
            if signs[nearest_ind] < 1.0 {
                // we hit the top/right/etc edge of the block and need to drop by 1;
                nearest_point[nearest_ind] -= 1.0;
            }

            hits.push(start.position_in_same_dimension(nearest_point)); // should be a yield later

            int_points[nearest_ind] += signs[nearest_ind];
            nearest_points[nearest_ind] = (int_points[nearest_ind] - start.position[nearest_ind]) * slope / slope[nearest_ind] + start.position;

            if dists.min_element().abs() >= (end.position.x - start.position.x).abs() {
                break;
            }
        }
        */
    }
}