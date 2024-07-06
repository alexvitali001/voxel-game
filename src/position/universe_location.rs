use std::ops::{Add, Sub, AddAssign};

use crate::chunk::chunk::CHUNK_SIZE_I32;
use bevy::{math::f64::DVec3, prelude::*};

#[derive(Default, Debug, Component, Clone, Copy)]
pub struct UniverseLocation {
    pub dimension: u32,
    pub position: DVec3 
}

impl UniverseLocation {
    pub fn from_dim_xyz<T>(dimension: u32, position: T) -> Self
        where T: Into<DVec3> {
        return Self {
            dimension: dimension,
            position: position.into()
        };
    }

    // get what chunk we are currently in
    pub fn get_chunk_position(&self) -> IVec3 {
        // society when integer division rounds towards zero and not down and i have to do this
        let chunk_size = CHUNK_SIZE_I32 as f64;
        return IVec3::new(
            (self.position.x / chunk_size).floor() as i32,
            (self.position.y / chunk_size).floor() as i32,
            (self.position.z / chunk_size).floor() as i32
        )
    }

    // get where within the chunk we are
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