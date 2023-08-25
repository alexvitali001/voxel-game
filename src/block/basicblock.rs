use block_mesh::VoxelVisibility;
use bevy::render::texture::Image;
use bevy::asset::Handle;

#[derive(Clone)]
pub enum BlockMaterial {
    Empty,
    Solid(Handle<Image>),
    #[allow(dead_code)]
    Fluid,
}

pub struct BasicBlock {
    pub name: String,
    pub mesh_visibility: VoxelVisibility,
    pub material_type: BlockMaterial,
}
