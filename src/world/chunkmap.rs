

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
        // println!("fetching {:?}", coords);
        if !self.cache.contains(&coords) {
            // println!("not in level one cache");
            let key = encode::to_vec(&coords)
                .expect("Serialiser could not serialise key");
            let serc;
            if !self.db.contains_key(key.as_slice())
                .expect("Sled DB failed to query for existence of key") {
                // println!("not in db, generating");
                let c = Chunk::generate_chunk(registry, chunk_x, chunk_y, chunk_z);
                serc = encode::to_vec(&c).expect("Could not serialise chunk");
                // println!("serialising");
                self.db.insert(key.as_slice(), serc)
                    .expect("Sled DB failed to insert");
                // println!("inserting into db");
            }
            // println!("updating level one cache");
            let val = self.db.get(key.as_slice())
                .expect("Sled DB encountered error")
                .expect("Chunk should be generated if not already present before this");
            // println!("fetched value from db");
            let c = decode::from_slice(&val)
                .expect("Deserialisation failed");
            // println!("deserialized");
            self.cache.put(coords, c);
        }
        // println!("found, returning\n");
        self.cache.get(&coords).expect("This chunk should be in the cache")
    }
}

impl Default for ChunkMap {
    fn default() -> Self {
        return ChunkMap::new();
    }
}