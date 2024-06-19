use bevy::prelude::*;
use crate::chunk::chunk::BlockId;
use std::collections::HashMap;
use super::universe::Universe;

#[derive(Resource)]
pub struct BlockMaterials {
    map: HashMap<(BlockId, u64), Handle<StandardMaterial>>
}

impl BlockMaterials {
    pub fn new() -> BlockMaterials {
        BlockMaterials {
            map: HashMap::new()
        }
    }

    pub fn get_material(&mut self, asset_server : &AssetServer, materials : &mut Assets<StandardMaterial>, universe: &Universe, id: BlockId, variant: u64) -> Handle<StandardMaterial> {
        match self.map.get(&(id, variant)) {
            Some(mat) => mat.clone(),
            None => {
                let block_data = universe.get_block_data_id(id);
                let h_img = asset_server.load(block_data.texture_file.clone());
                let sm = StandardMaterial {
                    perceptual_roughness: 0.95,
                    base_color_texture: Some(h_img),
                    ..default()
                };

                let h = materials.add(sm);
                self.map.insert((id, variant), h.clone());
                h
            }
        }
    }
}