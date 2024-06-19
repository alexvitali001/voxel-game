

use crate::chunk::chunk::Chunk;
use bevy::prelude::*;

use sled;
use std::env;
use sled::IVec;
use sled::MergeOperator;

use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[derive(AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
pub struct Coords {
    x: i32,
    y: i32,
    z: i32
}

#[derive(Resource, Clone)]
pub struct ChunkMap {
    db: sled::Db
}

impl ChunkMap {
    pub fn new() -> Self {
        let path = env::temp_dir().join("chunkworld");
        println!("{:?}", path);
        ChunkMap {
            db: sled::Config::default()
                .path(path)
                .use_compression(true)
                .compression_factor(5)
                .mode(sled::Mode::HighThroughput)
                .open().unwrap()
        }
    }

    pub fn flush_chunk(&self, coords: &IVec3, chunk: &Chunk) {
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
    ) -> Option<IVec> {
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
            // println!("penis wenis dick and balls");
            Some(val)
        }
    }

    pub fn fetch_chunk_exists(
        &self,
        coords: &IVec3,
    ) -> IVec {
        self.fetch_chunk(coords)
            .expect("there should be a chunk")
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        return ChunkMap::new();
    }
}