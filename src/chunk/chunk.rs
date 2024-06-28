use bevy::prelude::Component;

use zerocopy::{
        AsBytes, FromBytes, FromZeroes
    };

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBytes, FromZeroes, AsBytes)]
#[repr(C)]
pub struct BlockId(pub u32);

pub const AIR: BlockId = BlockId(0);

#[derive(Component, FromBytes, FromZeroes, AsBytes)]
#[repr(C)]
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
}
