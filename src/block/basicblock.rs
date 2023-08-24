use block_mesh::VoxelVisibility;
pub enum BlockMaterial {
    Empty,
    Solid,
    Fluid,
}

pub struct BasicBlock {
    pub name: String,
    pub mesh_visibility: VoxelVisibility,
    pub material_type: BlockMaterial,
}
