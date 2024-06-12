

use crate::block::chunk::Chunk;
use crate::block::blockregistry::BlockRegistry;
use bevy::prelude::*;

use rmp_serde::{decode, encode};

use sled;
use std::env;
use std::num::NonZeroUsize;
use lru::LruCache;

#[derive(Resource)]
pub struct ChunkMap {
    db: sled::Db,
    cache: LruCache<(i32, i32, i32), Chunk>
}

const LRU_CACHE_SIZE: usize = 4096;

impl ChunkMap {
    pub fn new() -> Self {
        let path = env::temp_dir().join("chunkworld");
        println!("{:?}", path);
        ChunkMap {
            db: sled::open(path)
                .expect("Database creation failed"),
            cache: LruCache::new(NonZeroUsize::new(LRU_CACHE_SIZE).unwrap())
        }
    }

    pub fn load_chunk(
        &mut self,
        registry: &BlockRegistry,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> &Chunk {
        let coords = (chunk_x, chunk_y, chunk_z);
        if !self.cache.contains(&coords) {
            let key = encode::to_vec(&coords)
                .expect("Serialiser could not serialise key");
            let c;
            if !self.db.contains_key(key.as_slice())
                .expect("Sled DB failed to query for existence of key") {
                c = Chunk::generate_chunk(registry, chunk_x, chunk_y, chunk_z);
            } else {
                let val = self.db.get(key.as_slice())
                    .expect("Sled DB encountered error")
                    .expect("Chunk should be generated if not already present before this");
                c = decode::from_slice(&val)
                    .expect("Deserialisation failed");
            }
            match self.cache.push(coords, c) {
                Some(res) => {
                    if coords != res.0 {
                        let ser_coords = encode::to_vec(&res.0)
                            .expect("Failed to serialize coords");
                        let ser_chunk = encode::to_vec(&res.1)
                            .expect("Could not serialise chunk");
                        self.db.insert(ser_coords, ser_chunk)
                            .expect("Sled DB failed to insert");
                    }
                }, None => {}
            };
        }
        self.cache.get(&coords).expect("This chunk should be in the cache")
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        return ChunkMap::new();
    }
}