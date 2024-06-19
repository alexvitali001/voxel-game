

use crate::chunk::chunk::BlockId;
use crate::chunk::chunk::Chunk;
use crate::world::block::BlockData;
use crate::world::block::BlockType;
use bevy::prelude::*;

use parking_lot::RwLock;
use sled;
use sled::Tree;
use std::env;
use std::sync::Arc;
use std::collections::HashMap;
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
pub struct Universe {
    db: sled::Db,
    block_registry_idmap: Arc<RwLock<HashMap<String, BlockId>>>,
    block_registry_datamap: Arc<RwLock<HashMap<BlockId, Arc<BlockData>>>>
}

fn new_registry<T, U>() -> Arc<RwLock<HashMap<T, U>>> {
    Arc::new(RwLock::new(HashMap::new()))
}

impl Universe {
    pub fn new() -> Self {
        let path = env::temp_dir().join("chunkworld");
        println!("{:?}", path);
        let u = Universe {
            db: sled::Config::default()
                .path(path)
                .use_compression(true)
                .compression_factor(5)
                .mode(sled::Mode::HighThroughput)
                .open().unwrap(),

            block_registry_idmap: new_registry(),
            block_registry_datamap: new_registry()
        };

        // guarantee a generic air for block ID 0
        u.register_block(BlockData {
            name: String::from("air"), 
            block_type: BlockType::Empty,
            texture_file: String::from("")
        });

        u
    }

    // CHUNK HANDLING
    pub fn dimension(&self, dim : &str) -> Tree {
        self.db.open_tree(&format!("dim:{}", dim))
               .expect(&format!("Could not load dimension {}", dim))
    }
    pub fn flush_chunk(&self, coords: &IVec3, chunk: &Chunk) {
        let coords = Coords {
            x: coords.x,
            y: coords.y,
            z: coords.z
        };
        let ser_coords = coords.as_bytes();
        let ser_chunk = chunk.as_bytes();
        self.dimension("overworld")
            .insert(ser_coords, ser_chunk)
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
        let dim = self.dimension("overworld");
        let key = coords.as_bytes();
        if !dim.contains_key(key)
            .expect("Sled DB failed to query for existence of key") {
            None
        } else {
            let val = dim.get(key)
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

    // BLOCK REGISTRY THINGS
    pub fn register_block(&self, block : BlockData) {
        let mut id_map = self.block_registry_idmap.write();
        let mut data_map = self.block_registry_datamap.write();

        let new_id = BlockId(data_map.len() as u32);
        id_map.insert(block.name.clone(), new_id);
        data_map.insert(new_id, Arc::new(block));
        
        // todo sync to file on write
    }

    pub fn get_block_data_id(&self, id: BlockId) -> Arc<BlockData> {
        self.block_registry_datamap.read()
                                   .get(&id)
                                   .expect(&format!("Failed to get block data for id {}", id.0))
                                   .clone()
    }

    pub fn get_block_data_name(&self, name: String) -> Arc<BlockData> {
        let m = self.block_registry_idmap.read();
        let id = m.get(&name)
                            .expect(&format!("Failed to get block id for name {}", name));
        self.get_block_data_id(*id)
    }

    pub fn block_id_from_name(&self, name: String) -> BlockId {
        self.block_registry_idmap.read()
                .get(&name)
                .expect(&format!("Failed to get block id for name {}", name))
                .clone()
    }

}

impl Default for Universe {
    fn default() -> Self {
        return Universe::new();
    }
}