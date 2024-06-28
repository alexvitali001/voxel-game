use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::*;
use crate::chunk::chunk::BlockId;
use crate::chunk::chunk::Chunk;
use crate::chunk::chunk::CHUNK_SIZE_I32;
use crate::chunk::mesh::bake;
use crate::WorldPosition;
use zerocopy::FromBytes;
use super::universe::Universe;
use super::block_materials::BlockMaterials;
use crate::terrain::terraingen::generate_chunk;

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
                    let c = generate_chunk(&u, coords);
                    u.flush_chunk(&coords, &c);
                    debug!("flushed chunk {} {} {}", coords.x, coords.y, coords.z);
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
#[derive(Event)]
pub struct LoadChunkEvent(pub IVec3);

fn on_load_chunk(
    mut ev_load : EventReader<LoadChunkEvent>,
    mut ev_remesh: EventWriter<ChunkRemeshEvent>,
    mut ev_gen: EventWriter<GenerateChunkEvent>,
    mut commands : Commands,
    universe: Res<Universe>
) {
    for ev in ev_load.read() {
        let coords = ev.0;
        if universe.chunk_generated(&coords) {
            // if the chunk was already generated, just spawn the entity and send a remesh event
            let e = commands.spawn((
                ChunkPosition(coords),
                ChunkMeshList(vec![])
            )).id();
            ev_remesh.send(ChunkRemeshEvent(coords, e));
        } else {
            // if not, send a generate event
            ev_gen.send(GenerateChunkEvent(coords));
        }
    }
}

#[derive(Event)]
pub struct UnloadChunkEvent(pub Entity);

fn on_unload_chunk(
    mut ev_unload : EventReader<UnloadChunkEvent>,
    mut commands : Commands,
    mesh_list_q:  Query<&ChunkMeshList>
) {
    for ev in ev_unload.read() { 
        // TODO: We should probably use bevy's builtin parent-child relationship here to child the meshes to the "Loaded Chunk" entity
        // that would probably make this less cumbersome. and a lot of the shit in this file really.

        let e = ev.0;
        for m in &mesh_list_q.get(e).unwrap().0 {
            // delete all the meshes
            commands.entity(*m).despawn();
        }
        commands.entity(e).despawn(); 
    }
}

const HORIZONTAL_RENDER_DISTANCE : i32 = 8;
const VERTICAL_RENDER_DISTANCE : i32 = 8;
use crate::player::*;
use std::collections::HashSet;
use std::cmp::max;
fn chunk_loading_manager(
    mut ev_load : EventWriter<LoadChunkEvent>,
    mut ev_unload : EventWriter<UnloadChunkEvent>,
    player_query: Query<&mut WorldPosition, With<ThisPlayer>>,
    chunk_query: Query<(Entity, &ChunkPosition, Option<&ChunkRemeshTask>, Option<&GenerateChunkTask>)>
                // we query the tasks so we can not avoid unloading chunks that have tasks on them
                // because unloading chunks that are being generated/meshed seems Like A Bad Idea
) {
    let player_chunk = player_query.single().get_chunk_position();

    let mut already_loaded : HashSet<IVec3> = HashSet::new();

    // unload chunks that are too far away in any direction
    for (e, pos, rt, gt) in &chunk_query {
        if rt.is_some() || gt.is_some() {
            // chunks with tasks are always considered "in bounds", so they aren't unloaded or loaded again
            already_loaded.insert(pos.0); 
        } else if (pos.0[1] - player_chunk[1]).abs() > VERTICAL_RENDER_DISTANCE {
            info!("Unloading chunk {},{},{} (outside vertical render distance)", pos.0[0], pos.0[1], pos.0[2]);
            ev_unload.send(UnloadChunkEvent(e));
        } else if max( // using chebyshev distance for now
                (pos.0[0] - player_chunk[0]).abs(), 
                (pos.0[2] - player_chunk[2]).abs()
            ) > HORIZONTAL_RENDER_DISTANCE { 
                info!("Unloading chunk {},{},{} (outside horizontal render distance)", pos.0[0], pos.0[1], pos.0[2]);
                ev_unload.send(UnloadChunkEvent(e));
        } else {
            // maintain a list of already loaded in-bound chunks so as not to reload them
            // info!("Sparing chunk {},{},{}", pos.0[0], pos.0[1], pos.0[2]);
            already_loaded.insert(pos.0);
        }
    }

    // check if new chunks need to be loaded
    // stupid dumb algorithm for doing this, in the future we want this to be a spiral to ensure the chunk you're in always goes into the task pool first
    for dx in -HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE {
        for dz in -HORIZONTAL_RENDER_DISTANCE..=HORIZONTAL_RENDER_DISTANCE {
            for dy in -VERTICAL_RENDER_DISTANCE..=VERTICAL_RENDER_DISTANCE {
                let coords = IVec3::new(dx,dy,dz) + player_chunk;
                if !already_loaded.contains(&coords) {
                    info!("Loading chunk {},{},{}", coords[0], coords[1], coords[2]);
                    ev_load.send(LoadChunkEvent(coords));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct ChunkEventsPlugin;

impl Plugin for ChunkEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                chunk_loading_manager,
                (on_load_chunk, on_unload_chunk),
                (finish_generating_tasks, on_generate_chunk).chain(),
                (finish_remeshing_tasks,on_chunk_remesh).chain(), 
                ).chain())
           .add_event::<GenerateChunkEvent>()
           .add_event::<ChunkRemeshEvent>()
           .add_event::<LoadChunkEvent>()
           .add_event::<UnloadChunkEvent>();
    }
}
