
use std::collections::HashMap;

use crate::block::chunk::Chunk;
use crate::block::blockregistry::BlockRegistry;
use bevy::prelude::*;

#[derive(Resource)]
pub struct ChunkWorld {
    db: HashMap<(i32, i32, i32), Chunk>
}

impl ChunkWorld {
    pub fn new() -> Self {
        ChunkWorld {
            db: HashMap::new()
        }
    }

    pub fn load_chunk(
        &mut self,
        registry: &BlockRegistry,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> &Chunk {
        if !self.db.contains_key(&(chunk_x, chunk_y, chunk_z)) {
            let c = Chunk::generate_chunk(registry, chunk_x, chunk_y, chunk_z);
            self.db.insert((chunk_x, chunk_y, chunk_z),c);
        } 
        self.db.get(&(chunk_x, chunk_y, chunk_z))
            .expect("Chunk should be generated if not already present before this")
    }
}

impl Default for ChunkWorld {
    fn default() -> Self {
        return ChunkWorld::new();
    }
}