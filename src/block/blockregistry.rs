use crate::block::basicblock::BasicBlock;
use crate::block::chunk::BlockId;
use crate::BlockMaterial::*;
use std::collections::HashMap;

use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::utils::default;

#[derive(Resource)]
pub struct BlockRegistry {
    registered_blocks: Vec<BasicBlock>,
    name_map: HashMap<String, BlockId>,
    block_materials: HashMap<BlockId, Handle<StandardMaterial>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        BlockRegistry {
            registered_blocks: vec![],
            name_map: HashMap::new(),
            block_materials: HashMap::new(),
        }
    }

    pub fn register_block(
        &mut self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        block: BasicBlock,
    ) {
        let new_block_id = self.registered_blocks.len() as u32;
        self.name_map
            .insert(block.name.clone(), BlockId(new_block_id));

        if let Solid(h_img) = block.material_type.clone() {
            self.block_materials.insert(
                BlockId(new_block_id),
                materials.add(StandardMaterial {
                    perceptual_roughness: 0.95,
                    base_color_texture: Some(h_img),
                    ..default()
                }),
            );
        }

        self.registered_blocks.push(block);
    }

    #[allow(dead_code)]
    pub fn visibility_from_id(&self, id: BlockId) -> block_mesh::VoxelVisibility {
        let BlockId(n) = id;
        return self.registered_blocks[n as usize].mesh_visibility;
    }

    pub fn id_from_name(&self, name: String) -> Option<BlockId> {
        return self.name_map.get(&name).copied();
    }
    pub fn block_from_id(&self, id: BlockId) -> &BasicBlock {
        let BlockId(n) = id;
        return &self.registered_blocks[n as usize];
    }

    pub fn material_from_id(&self, id: &BlockId) -> Option<&Handle<StandardMaterial>> {
        self.block_materials.get(id)
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        return BlockRegistry::new();
    }
}
