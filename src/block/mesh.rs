use crate::block::chunk::BlockId;
use bevy::{
    prelude::Mesh,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use block_mesh::{
    greedy_quads,
    ndshape::{ConstShape, ConstShape3u32},
    GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};

use super::{
    blockregistry::BlockRegistry,
    chunk::{Chunk, AIR},
};

struct MeshVoxel {
    id: BlockId,
    vis: VoxelVisibility,
}

const AIRVOXEL: MeshVoxel = MeshVoxel {
    id: AIR,
    vis: VoxelVisibility::Empty,
};

impl Voxel for MeshVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        return self.vis;
    }
}

impl MergeVoxel for MeshVoxel {
    type MergeValue = BlockId;
    fn merge_value(&self) -> Self::MergeValue {
        return self.id;
    }

    type MergeValueFacingNeighbour = Self::MergeValue;
    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        return self.merge_value();
    }
}

const CHUNK_MESH_SIZE: u32 = 32 + 2;
type ChunkMeshShape = ConstShape3u32<CHUNK_MESH_SIZE, CHUNK_MESH_SIZE, CHUNK_MESH_SIZE>;

pub fn bake(registry: &BlockRegistry, chunk: &Chunk) -> Mesh {
    let mut voxels = [AIRVOXEL; ChunkMeshShape::SIZE as usize];

    for i in 0..ChunkMeshShape::SIZE {
        let [x, y, z] = ChunkMeshShape::delinearize(i);
        if x == 0 || y == 0 || z == 0 || 
           x == CHUNK_MESH_SIZE - 1 || y == CHUNK_MESH_SIZE - 1 || z == CHUNK_MESH_SIZE - 1 {
            continue;
        }
        let block_id = chunk.get(x-1, y-1, z-1);
        let block = registry.block_from_id(block_id);
        voxels[i as usize] = MeshVoxel {
            id: block_id,
            vis: block.mesh_visibility,
        };
    }
    let mut buffer = GreedyQuadsBuffer::new(voxels.len());
    greedy_quads(
        &voxels,
        &ChunkMeshShape {},
        [0; 3],
        [CHUNK_MESH_SIZE - 1; 3],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);
    // let mut data = Vec::with_capacity(num_vertices);
    let scale = 1.0;

    //normal face index depends on the quad orientation config
    for (block_face_normal_index, (group, face)) in buffer
        .quads
        .groups
        .as_ref()
        .iter()
        .zip(RIGHT_HANDED_Y_UP_CONFIG.faces.iter())
        .enumerate()
    {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(quad, scale)
                                             .map(|q| q.map(|x| x - 1.0)));
            normals.extend_from_slice(&face.quad_mesh_normals());
            let normal = &face.quad_mesh_normals()[0];
            let [u,v] = [quad.width as f32, quad.height as f32];

            let uv = if normal[2] - normal[0] + normal[1] > 0.0 {
                [[0.0, v], [u, v], [0.0, 0.0], [u, 0.0]]
            } else {
                [[u, v], [0.0, v], [u, 0.0], [0.0, 0.0]]
            };
            uvs.extend_from_slice(&uv);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uvs),
    );

    //todo: in the future we might want to encode all the information onto a single uint32
    // mesh.insert_attribute(
    //    VoxelTerrainMesh::ATTRIBUTE_DATA,
    //    VertexAttributeValues::Uint32(data),
    // );

    mesh.set_indices(Some(Indices::U32(indices)));

    return mesh;
}
