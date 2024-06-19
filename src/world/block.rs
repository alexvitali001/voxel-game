use block_mesh::VoxelVisibility;


pub enum BlockType {
    Empty,
    OpaqueSolid,
    TranslucentSolid,
    Fluid,
}

impl BlockType {
    pub fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Self::Empty => VoxelVisibility::Empty,
            Self::OpaqueSolid => VoxelVisibility::Opaque,
            Self::TranslucentSolid => VoxelVisibility::Translucent,
            Self::Fluid => VoxelVisibility::Translucent
        }
    }
}

pub struct BlockData {
    pub name: String,
    pub block_type: BlockType,
    pub texture_file: String
}
