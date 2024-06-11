use crate::block::blockregistry::BlockRegistry;
use noise::NoiseFn;
use noise::Perlin;
use bevy::prelude::Component;

use serde::{Serialize, Deserialize};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub u32);

pub const AIR: BlockId = BlockId(0);

#[derive(Component, Serialize, Deserialize)]
pub struct Chunk {
    blocks: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        return Chunk {
            blocks: [[[AIR; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        };
    }

    pub fn place(&mut self, block: BlockId, pos: (u32, u32, u32)) {
        let (x, y, z) = pos;
        self.blocks[x as usize][z as usize][y as usize] = block;
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> BlockId {
        return self.blocks[x as usize][z as usize][y as usize];
    }

    pub fn generate_chunk(
        registry: &BlockRegistry,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Chunk {
        let noise = Perlin::new(0);
        let mut chunk = Chunk::new();

        let stone = registry.id_from_name(String::from("stone")).unwrap();
        let dirt = registry.id_from_name(String::from("dirt")).unwrap();
        for relative_x in 0..CHUNK_SIZE_I32 {
            for relative_z in 0..CHUNK_SIZE_I32 {
                // get world coordinates of this column
                let x = chunk_x * CHUNK_SIZE_I32 + relative_x;
                let z = chunk_z * CHUNK_SIZE_I32 + relative_z;

                // convert them to f64 and scale to put in noise
                let smoothness = 10.0;
                let amplitude = 10.0;
                let noise_x = x as f64 / smoothness;
                let noise_z = z as f64 / smoothness;

                // maximum height of surface
                let height = (5.0 + amplitude * noise.get([noise_x, noise_z])).ceil() as i32;
                let chunk_floor = chunk_y * CHUNK_SIZE_I32;

                // skip this column if the height is too low
                if chunk_floor > height {
                    continue;
                }
                let top = std::cmp::min(height - chunk_floor, CHUNK_SIZE_I32);
                for relative_y in 0..top - 1 {
                    chunk.place(
                        stone,
                        (relative_x as u32, relative_y as u32, relative_z as u32),
                    );
                }
                if top > 0 {
                    chunk.place(
                        dirt,
                        (relative_x as u32, (top - 1) as u32, relative_z as u32),
                    );
                }
            }
        }
        return chunk;
    }
}
