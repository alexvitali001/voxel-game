use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::*;
use crate::chunk::chunk::BlockId;
use crate::chunk::chunk::Chunk;
use crate::chunk::mesh::bake;
use crate::WorldPosition;
use zerocopy::FromBytes;
use super::universe::Universe;
use super::block_materials::BlockMaterials;

#[derive(Component)]
pub struct ChunkPosition(pub IVec3);

#[derive(Component)]
pub struct ChunkMeshList(pub Vec<Entity>);

#[derive(Bundle)]
pub struct UngeneratedChunkBundle {
    pub chunk_position: ChunkPosition,
    pub meshes: ChunkMeshList,
    pub task: GenerateChunkTask
}

#[derive(Event)]
pub struct GenerateChunkEvent(pub IVec3);

#[derive(Component)]
pub struct GenerateChunkTask(pub Task<()>);

fn on_generate_chunk(
    mut ev_gen : EventReader<GenerateChunkEvent>,
    mut commands : Commands,
    universe: Res<Universe>
) {
    let task_pool = AsyncComputeTaskPool::get();

    for ev in ev_gen.read() {
        let coords = ev.0;
        // debug!("generating {} {} {}", coords.x, coords.y, coords.z);
        let u = (*universe.as_ref()).clone();
        commands.spawn(
            UngeneratedChunkBundle {
                chunk_position: ChunkPosition(coords),
                meshes: ChunkMeshList(Vec::new()), 
                task: GenerateChunkTask(task_pool.spawn(async move {
                    let c = Chunk::generate_chunk(&u, coords);
                    u.flush_chunk(&coords, &c);
                    // debug!("flushed chunk {} {} {}", coords.x, coords.y, coords.z);
                }))});
    }
}

fn finish_generating_tasks(
    mut chunk_query: Query<(Entity, &ChunkPosition, &mut GenerateChunkTask)>,
    mut commands: Commands,
    mut ev_remesh: EventWriter<ChunkRemeshEvent>
) {
    chunk_query.iter_mut()
        .for_each(|(entity, ChunkPosition(pos), task)| {
        if task.0.is_finished() {
            // delete the task and get the entity id
            let ce = commands.entity(entity)
                    .remove::<GenerateChunkTask>()
                    .id();

            // fire remesh event
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
    universe: Res<Universe>
) {
    let task_pool = AsyncComputeTaskPool::get();

    for ChunkRemeshEvent(pos, e) in ev_remesh.read() {
        let u = (*universe.as_ref()).clone();
        let c = u.fetch_chunk_exists(pos);
        //let p = pos.clone();
        commands.entity(*e).insert(
            ChunkRemeshTask(task_pool.spawn(async move {
                //debug!("remeshing {} {} {}", p.x, p.y, p.z);
                let mm = bake(
                    &u,
                    Chunk::ref_from(c.as_ref()).unwrap()
                );
                //debug!("done remeshing {} {} {}", p.x, p.y, p.z);
                mm
            }))
        );
    }
}

fn finish_remeshing_tasks(
    mut chunk_query: Query<(Entity, &mut ChunkMeshList, &ChunkPosition, &mut ChunkRemeshTask)>,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut block_materials: ResMut<BlockMaterials>,
    universe: Res<Universe>,
    //player: Query<&WorldPosition, With<ThisPlayer>>,
    asset_server: Res<AssetServer>,
) {
    //let pwp = player.single();
    chunk_query.iter_mut()
        .for_each(|(entity, mut mesh_list, ChunkPosition(pos), mut task)| {
            if let Some(new_meshes) = block_on(poll_once(&mut task.0)) {
                // delete all previous meshes
                // does despawning the entity automatically unload the mesh asset in Assets<Mesh>?
                // is that something we need to worry about?
                // if unloading isnt automatic, this leaks memory. Too Bad!
                for m in mesh_list.0.drain(..) {
                    commands.entity(m).despawn();
                }

                for (bid, mesh) in new_meshes {
                    let mat = block_materials.get_material(asset_server.as_ref(), material_assets.as_mut(), &universe, bid, 0,);
                    //wp.to_render_transform(pwp, &mut trans);
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
                    mesh_list.0.push(e);
                }
                // update the mesh list
                commands.entity(entity).remove::<ChunkRemeshTask>();
            }
        })
}

#[derive(Component)]
pub struct ChunkEventsPlugin;

impl Plugin for ChunkEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                (finish_remeshing_tasks,on_chunk_remesh).chain(), 
                (finish_generating_tasks, on_generate_chunk).chain()))
           .add_event::<GenerateChunkEvent>()
           .add_event::<ChunkRemeshEvent>();
    }
}
