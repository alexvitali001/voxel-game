

use crate::block::chunk::Chunk;
use crate::block::blockregistry::BlockRegistry;
use bevy::prelude::*;

use rmp_serde::{decode, encode};

use sled;
use std::env;

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

    pub fn flush_chunk(&mut self, coords: &(i32, i32, i32), chunk: &Chunk) {
        let ser_coords = encode::to_vec(coords)
            .expect("Failed to serialize coords");
        let ser_chunk = encode::to_vec(chunk)
            .expect("Could not serialise chunk");
        self.db.insert(ser_coords, ser_chunk)
            .expect("Sled DB failed to insert");
    }

    pub fn load_chunk(
        &mut self,
        registry: &BlockRegistry,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Option<Chunk> {
        let coords = (chunk_x, chunk_y, chunk_z);
        let key = encode::to_vec(&coords)
            .expect("Serialiser could not serialise key");
        if !self.db.contains_key(key.as_slice())
            .expect("Sled DB failed to query for existence of key") {
            // TODO: impl chunk generation here!!
            None
        } else {
            let val = self.db.get(key.as_slice())
                .expect("Sled DB encountered error")
                .expect("Chunk should be generated if not already present before this");
            Some(decode::from_slice(&val)
                .expect("Deserialisation failed"))
        }
    }

    pub fn load_chunk_exists(
        &mut self,
        registry: &BlockRegistry,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Chunk {
        self.load_chunk(registry, chunk_x, chunk_y, chunk_z)
            .expect("there should be a chunk")
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        return ChunkMap::new();
    }
}