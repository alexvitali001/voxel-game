use crate::block::basicblock::BasicBlock;
use crate::block::basicblock::BlockMaterial;
use crate::block::chunk::BlockId;
use block_mesh::VoxelVisibility;
use std::collections::HashMap;
pub struct BlockRegistry {
    registered_blocks: Vec<BasicBlock>,
    name_map: HashMap<String, BlockId>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut registry = BlockRegistry {
            registered_blocks: vec![],
            name_map: HashMap::new(),
        };

        registry.register_block(BasicBlock {
            name: String::from("air"),
            mesh_visibility: VoxelVisibility::Empty,
            material_type: BlockMaterial::Empty,
        });

        registry.register_block(BasicBlock {
            name: String::from("stone"),
            mesh_visibility: VoxelVisibility::Opaque,
            material_type: BlockMaterial::Solid,
        });

        registry.register_block(BasicBlock {
            name: String::from("dirt"),
            mesh_visibility: VoxelVisibility::Opaque,
            material_type: BlockMaterial::Solid,
        });
        return registry;
    }

    pub fn register_block(&mut self, block: BasicBlock) {
        let new_block_id = self.registered_blocks.len() as u32;
        self.name_map
            .insert(block.name.clone(), BlockId(new_block_id));
        self.registered_blocks.push(block);
    }

    pub fn id_from_name(&self, name: String) -> Option<BlockId> {
        return self.name_map.get(&name).copied();
    }
    pub fn block_from_id(&self, id: BlockId) -> &BasicBlock {
        let BlockId(n) = id;
        return &self.registered_blocks[n as usize];
    }
}
