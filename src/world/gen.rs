use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::*;
use crate::block::blockregistry::BlockRegistry;
use crate::block::chunk::BlockId;
use crate::registryresource::RegistryResource;
use crate::block::chunk::Chunk;
use crate::block::mesh::bake;
use crate::WorldPosition;
use futures_lite::future;
use crate::world::chunkmap::ChunkMap;

#[derive(Component)]
pub struct ChunkPosition(IVec3);

#[derive(Component)]
pub struct ChunkMeshList(Vec<Entity>);

#[derive(Bundle)]
pub struct UngeneratedChunkBundle {
    chunk_position: ChunkPosition,
    meshes: ChunkMeshList,
    task: GenerateChunkTask
}

#[derive(Event)]
pub struct GenerateChunkEvent(pub IVec3);

#[derive(Component)]
pub struct GenerateChunkTask(Task<Chunk>);

fn on_generate_chunk(
    mut ev_gen : EventReader<GenerateChunkEvent>,
    mut commands : Commands,
    block_registry_resource: Res<RegistryResource<BlockRegistry>>
) {
    let task_pool = AsyncComputeTaskPool::get();

    for ev in ev_gen.read() {
        let coords = ev.0;
        let br_arc = block_registry_resource.clone_registry();
        println!("generating {} {} {}", coords.x, coords.y, coords.z);
        commands.spawn(
            UngeneratedChunkBundle {
                chunk_position: ChunkPosition(coords),
                meshes: ChunkMeshList(Vec::new()), 
                task: GenerateChunkTask(task_pool.spawn(async move {
                    Chunk::generate_chunk(&br_arc.read(), coords.x, coords.y, coords.z)
                }))});
    }
}

fn finish_generating_tasks(
    mut chunk_query: Query<(Entity, &ChunkPosition, &mut GenerateChunkTask)>,
    mut commands: Commands,
    mut ev_remesh: EventWriter<ChunkRemeshEvent>,
    chunkmap_resource: Res<RegistryResource<ChunkMap>>
) {
    let cm_arc = chunkmap_resource.clone_registry();
    chunk_query.iter_mut()
        .for_each(|(entity, ChunkPosition(pos), mut task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut task.0)) {
            // write the chunk to the database
            let mut chunkmap = cm_arc.write();
            chunkmap.flush_chunk(&(pos.x, pos.y, pos.z), &chunk);
            println!("flushed chunk {} {} {}", pos.x, pos.y, pos.z);

            // add the chunk data to the chunk component and delete the task that generated it
            let ce = commands.entity(entity)
                    .remove::<GenerateChunkTask>()
                    .id();

            ev_remesh.send(ChunkRemeshEvent(*pos, ce));
            
        }
    });
}

#[derive(Event)]
pub struct ChunkRemeshEvent(pub IVec3, pub Entity);


#[derive(Component)]
pub struct ChunkRemeshTask(Task<HashMap<BlockId, Mesh>>);

fn on_chunk_remesh(
    mut ev_remesh : EventReader<ChunkRemeshEvent>,
    mut commands : Commands,
    block_registry_resource: Res<RegistryResource<BlockRegistry>>,
    chunkmap_resource: Res<RegistryResource<ChunkMap>>
) {

    let cm_arc = chunkmap_resource.clone_registry();
    let chunkmap = cm_arc.read();

    let task_pool = AsyncComputeTaskPool::get();

    for ChunkRemeshEvent(pos, e) in ev_remesh.read() {
        println!("grabbing {} {} {}", pos.x, pos.y, pos.z);
        let c = chunkmap.fetch_chunk_exists(pos.x, pos.y, pos.z);
        let br_arc = block_registry_resource.clone_registry();
        println!("remeshing {} {} {}", pos.x, pos.y, pos.z);
        commands.entity(*e).insert(
            ChunkRemeshTask(task_pool.spawn(async move {
                bake(&br_arc.read(), &c)
            }))
        );
    }
}

fn finish_remeshing_tasks(
    mut chunk_query: Query<(Entity, &mut ChunkMeshList, &ChunkPosition, &mut ChunkRemeshTask)>,
    mut commands: Commands,
    block_registry_resource: Res<RegistryResource<BlockRegistry>>,
    mut mesh_assets: ResMut<Assets<Mesh>>
) {
    let br_arc = block_registry_resource.clone_registry();
    let block_registry = br_arc.read();

    chunk_query.iter_mut()
        .for_each(|(entity, mut old_meshes, ChunkPosition(pos), mut task)| {
            if let Some(new_meshes) = future::block_on(future::poll_once(&mut task.0)) {
                // delete all previous meshes
                // does despawning the entity automatically unload the mesh asset in Assets<Mesh>?
                // is that something we need to worry about?
                // if unloading isnt automatic, this leaks memory. Too Bad!
                for m in old_meshes.0.drain(..) {
                    commands.entity(m).despawn();
                }

                // spawn new meshes and record their ids in the mesh list
                let mut mesh_list: Vec<Entity> = Vec::new();

                for (bid, mesh) in new_meshes {
                    if let Some(mat) = block_registry.material_from_id(&bid) {
                        let e = commands
                            .spawn(PbrBundle {
                                mesh: mesh_assets.add(mesh),
                                material: mat.clone(),
                                ..default()
                            })
                            .insert(WorldPosition::from_xyz(
                                (32 * pos.x) as f64,
                                (32 * pos.y) as f64,
                                (32 * pos.z) as f64,
                            )).id();
                        mesh_list.push(e);
                    }
                }

                // update the mesh list
                commands.entity(entity).insert(ChunkMeshList(mesh_list));
            }
        })
}

#[derive(Component)]
pub struct ChunkEventsPlugin;

impl Plugin for ChunkEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_generate_chunk, finish_generating_tasks, on_chunk_remesh, finish_remeshing_tasks))
           .add_event::<GenerateChunkEvent>()
           .add_event::<ChunkRemeshEvent>();
    }
}