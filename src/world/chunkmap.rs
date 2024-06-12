

use crate::block::chunk::Chunk;
use bevy::prelude::*;

use sled;
use std::env;

use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[derive(AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
pub struct Coords {
    x: i32,
    y: i32,
    z: i32
}

#[derive(Resource)]
pub struct ChunkMap {
    db: sled::Db
}

impl ChunkMap {
    pub fn new() -> Self {
        let path = env::temp_dir().join("chunkworld");
        println!("{:?}", path);
        ChunkMap {
            db: sled::open(path)
                .expect("Database creation failed")
        }
    }

    pub fn flush_chunk(&mut self, coords: &IVec3, chunk: &Chunk) {
        let coords = Coords {
            x: coords.x,
            y: coords.y,
            z: coords.z
        };
        let ser_coords = coords.as_bytes();
        let ser_chunk = chunk.as_bytes();
        self.db.insert(ser_coords, ser_chunk)
            .expect("Sled DB failed to insert");
    }

    pub fn fetch_chunk(
        &self,
        coords: &IVec3,
    ) -> Option<Chunk> {
        let coords = Coords {
            x: coords.x,
            y: coords.y,
            z: coords.z
        };
        let key = coords.as_bytes();
        if !self.db.contains_key(key)
            .expect("Sled DB failed to query for existence of key") {
            None
        } else {
            let val = self.db.get(key)
                .expect("Sled DB encountered error")
                .expect("Chunk should be generated if not already present before this");
            println!("penis wenis dick and balls");
            Chunk::read_from(val.as_ref())
        }
    }

    pub fn fetch_chunk_exists(
        &self,
        coords: &IVec3,
    ) -> Chunk {
        self.fetch_chunk(coords)
            .expect("there should be a chunk")
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        return ChunkMap::new();
    }
}