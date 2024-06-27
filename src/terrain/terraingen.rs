use bevy_math::IVec3;

use crate::chunk::chunk::*;
use crate::world::universe::Universe;


pub fn generate_chunk(
    u: &Universe,
    coords: IVec3,
) -> Chunk {
    let mut chunk = Chunk::new();
    let chunk_x = coords[0];
    let chunk_y = coords[1];
    let chunk_z = coords[2];
    let stone = u.block_id_from_name(String::from("stone"));
    let dirt = u.block_id_from_name(String::from("dirt"));
    for relative_x in 0..CHUNK_SIZE_I32 {
        for relative_z in 0..CHUNK_SIZE_I32 {
            // get world coordinates of this column
            let x = chunk_x * CHUNK_SIZE_I32 + relative_x;
            let z = chunk_z * CHUNK_SIZE_I32 + relative_z;
            
            let top = u.dimension_noise.get_cont(x, z) as i32;

            // shitty impl, will hopefully eventually be density based
            for relative_y in 0..CHUNK_SIZE_I32 {
                let y = chunk_y * CHUNK_SIZE_I32 + relative_y;
                if y  < top{
                    chunk.place(
                        stone,
                        (relative_x as u32, relative_y as u32, relative_z as u32),
                    );
                } else if y == top {
                    chunk.place(
                        dirt,
                        (relative_x as u32, relative_y as u32, relative_z as u32),
                    );                   
                }
            }
        }
    }
    return chunk;
}