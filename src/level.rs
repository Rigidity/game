mod generator;

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap, utils::HashSet};
use bevy_tokio_tasks::TokioTasksRuntime;
use generator::LevelGenerator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::SqlitePool;
use tokio::time::sleep;

use crate::{
    block::Block,
    chunk::Chunk,
    game_state::GameState,
    loader::{BlockInteraction, GlobalTextureArray, VoxelMaterial},
    player::{Player, PlayerCamera},
    position::{BlockPos, ChunkPos},
};

const CHUNK_UNLOAD_DISTANCE: i32 = 10; // Should be larger than generation radius
const CHUNK_GENERATION_BATCH_SIZE: usize = 50; // Adjust this value as needed

#[derive(Debug, Clone, Copy)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Level::new())
            .insert_resource(LevelGenerator::new(42))
            .insert_resource(ChunkGenerationQueue::default())
            .add_systems(OnEnter(GameState::Setup), setup_level)
            .add_systems(
                OnEnter(GameState::Playing),
                (start_chunk_generation, start_saving),
            )
            .add_systems(
                Update,
                (build_chunk_meshes, unload_distant_chunks)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone)]
struct LoadedChunk {
    chunk: Chunk,
    entity: Entity,
}

#[derive(Resource)]
struct LevelDatabase(SqlitePool);

#[derive(Debug, Default, Clone, Resource)]
pub struct Level {
    chunks: HashMap<ChunkPos, LoadedChunk>,
}

impl Level {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos).map(|loaded| &loaded.chunk)
    }

    pub fn chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos).map(|loaded| &mut loaded.chunk)
    }

    pub fn entity(&self, pos: ChunkPos) -> Option<Entity> {
        self.chunks.get(&pos).map(|loaded| loaded.entity)
    }

    pub fn block(&self, pos: BlockPos) -> Block {
        let chunk_pos = pos.chunk_pos();
        let local_pos = pos.local_pos();
        self.chunk(chunk_pos)
            .map(|chunk| chunk.get(local_pos))
            .unwrap_or(Block::Air)
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Dirty;

#[derive(Debug, Clone, Copy, Component)]
pub struct Modified;

#[derive(Debug, Default, Resource)]
struct ChunkGenerationQueue {
    pending: HashSet<ChunkPos>,
}

fn setup_level(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(|mut ctx| async move {
        let pool = SqlitePool::connect("sqlite://./level.sqlite?mode=rwc")
            .await
            .unwrap();

        sqlx::migrate!().run(&pool).await.unwrap();

        ctx.run_on_main_thread(move |ctx| {
            ctx.world.insert_resource(LevelDatabase(pool));
            ctx.world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::Playing);
        })
        .await;
    });
}

fn start_chunk_generation(
    texture_array: Res<GlobalTextureArray>,
    db: Res<LevelDatabase>,
    runtime: Res<TokioTasksRuntime>,
    generator: Res<LevelGenerator>,
) {
    let texture_array = texture_array.clone();
    let db = db.0.clone();
    let mut generator = generator.clone();

    runtime.spawn_background_task(|mut ctx| async move {
        loop {
            let chunk_positions = ctx
                .run_on_main_thread(|ctx| {
                    let mut player_query = ctx.world.query_filtered::<&Transform, With<Player>>();
                    let level = ctx.world.resource::<Level>();
                    let queue = ctx.world.resource::<ChunkGenerationQueue>();

                    let player_transform = match player_query.iter(ctx.world).next() {
                        Some(transform) => transform,
                        None => return Vec::new(),
                    };

                    let player_pos = player_transform.translation;
                    let player_chunk = BlockPos::from_world(player_pos).chunk_pos();

                    // Find closest non-generated chunks within radius
                    let radius = 8;

                    let mut chunks = Vec::new();

                    for x in -radius..=radius {
                        for y in -radius..=radius {
                            for z in -radius..=radius {
                                let pos = ChunkPos::new(
                                    player_chunk.x + x,
                                    player_chunk.y + y,
                                    player_chunk.z + z,
                                );

                                // Calculate actual world-space distance from player
                                let dx = (pos.x as f32 * 16.0) - player_pos.x;
                                let dy = (pos.y as f32 * 16.0) - player_pos.y;
                                let dz = (pos.z as f32 * 16.0) - player_pos.z;
                                let distance_sq = (dx * dx + dy * dy + dz * dz) as i32;

                                if distance_sq > radius * radius * 256 {
                                    // Adjust radius for chunk size
                                    continue;
                                }

                                if level.chunk(pos).is_some() || queue.pending.contains(&pos) {
                                    continue;
                                }

                                chunks.push((distance_sq, pos));
                            }
                        }
                    }

                    // Sort by distance and take the closest N chunks
                    chunks.sort_by_key(|&(dist, _)| dist);
                    chunks.truncate(CHUNK_GENERATION_BATCH_SIZE);
                    chunks.into_iter().map(|(_, pos)| pos).collect()
                })
                .await;

            if chunk_positions.is_empty() {
                sleep(Duration::from_millis(500)).await;
                continue;
            }

            // Mark chunks as pending
            let chunk_positions_2 = chunk_positions.clone();
            ctx.run_on_main_thread(move |ctx| {
                let mut queue = ctx.world.resource_mut::<ChunkGenerationQueue>();
                for &pos in &chunk_positions_2 {
                    queue.pending.insert(pos);
                }
            })
            .await;

            // Generate chunks in parallel
            let mut chunks = Vec::new();
            for &chunk_pos in &chunk_positions {
                // Generate the chunk
                let chunk = match sqlx::query!(
                    "SELECT data FROM chunks WHERE x = ? AND y = ? AND z = ?",
                    chunk_pos.x,
                    chunk_pos.y,
                    chunk_pos.z
                )
                .fetch_optional(&db)
                .await
                .unwrap()
                {
                    Some(row) => bincode::deserialize(&row.data).unwrap(),
                    None => {
                        let chunk = generator.generate_chunk(chunk_pos);
                        let data = bincode::serialize(&chunk).unwrap();
                        sqlx::query!(
                            "INSERT INTO chunks (x, y, z, data) VALUES (?, ?, ?, ?)",
                            chunk_pos.x,
                            chunk_pos.y,
                            chunk_pos.z,
                            data
                        )
                        .execute(&db)
                        .await
                        .unwrap();
                        chunk
                    }
                };
                chunks.push((chunk_pos, chunk));
            }

            let texture_array = texture_array.clone();

            // Add the chunks to the world
            ctx.run_on_main_thread(move |ctx| {
                for (chunk_pos, chunk) in chunks {
                    let material =
                        ctx.world
                            .resource_mut::<Assets<VoxelMaterial>>()
                            .add(VoxelMaterial {
                                array_texture: texture_array.textures.clone(),
                                destroy_texture: texture_array.destroy.clone(),
                                block_interaction: BlockInteraction::default(),
                            });

                    let entity = ctx
                        .world
                        .spawn((
                            chunk_pos,
                            Dirty,
                            MeshMaterial3d(material),
                            Transform::from_xyz(
                                chunk_pos.x as f32 * 16.0,
                                chunk_pos.y as f32 * 16.0,
                                chunk_pos.z as f32 * 16.0,
                            ),
                        ))
                        .id();

                    // Get all neighboring chunk entities in one pass
                    let mut neighbor_entities = Vec::new();
                    {
                        let level = ctx.world.resource::<Level>();
                        for dx in -1..=1 {
                            for dy in -1..=1 {
                                for dz in -1..=1 {
                                    if dx == 0 && dy == 0 && dz == 0 {
                                        continue;
                                    }

                                    let neighbor_pos = ChunkPos::new(
                                        chunk_pos.x + dx,
                                        chunk_pos.y + dy,
                                        chunk_pos.z + dz,
                                    );

                                    if let Some(neighbor) = level.entity(neighbor_pos) {
                                        neighbor_entities.push(neighbor);
                                    }
                                }
                            }
                        }
                    }

                    // Insert the new chunk
                    ctx.world
                        .resource_mut::<Level>()
                        .chunks
                        .insert(chunk_pos, LoadedChunk { chunk, entity });

                    // Mark all neighbors as dirty in a single pass
                    for &neighbor in &neighbor_entities {
                        ctx.world.entity_mut(neighbor).insert(Dirty);
                    }

                    ctx.world
                        .resource_mut::<ChunkGenerationQueue>()
                        .pending
                        .remove(&chunk_pos);
                }
            })
            .await;

            sleep(Duration::from_millis(100)).await;
        }
    });
}

fn start_saving(db: Res<LevelDatabase>, runtime: Res<TokioTasksRuntime>) {
    let db = db.0.clone();

    runtime.spawn_background_task(move |mut ctx| async move {
        let player = sqlx::query!("SELECT x, y, z, roll, pitch, yaw FROM player")
            .fetch_one(&db)
            .await
            .unwrap();

        let player_pos = Vec3::new(player.x as f32, player.y as f32, player.z as f32);
        let player_rotation = Quat::from_euler(
            EulerRot::XYZ,
            player.roll as f32,
            player.pitch as f32,
            player.yaw as f32,
        );

        ctx.run_on_main_thread(move |ctx| {
            ctx.world
                .query_filtered::<&mut Transform, With<Player>>()
                .single_mut(ctx.world)
                .translation = player_pos;
            ctx.world
                .query_filtered::<&mut Transform, With<PlayerCamera>>()
                .single_mut(ctx.world)
                .rotation = player_rotation;
        })
        .await;

        loop {
            let (chunks_to_save, player_pos, player_rotation) = ctx
                .run_on_main_thread(|ctx| {
                    let modified: Vec<(Entity, ChunkPos)> = ctx
                        .world
                        .query_filtered::<(Entity, &ChunkPos), With<Modified>>()
                        .iter(ctx.world)
                        .map(|(entity, chunk_pos)| (entity, *chunk_pos))
                        .collect();

                    for (entity, _pos) in &modified {
                        ctx.world.entity_mut(*entity).remove::<Modified>();
                    }

                    let level = ctx.world.resource::<Level>();

                    let chunks: Vec<(ChunkPos, Chunk)> = modified
                        .iter()
                        .filter_map(|(_, pos)| Some((*pos, level.chunk(*pos)?.clone())))
                        .collect();

                    let player_pos = ctx
                        .world
                        .query_filtered::<&Transform, With<Player>>()
                        .single(ctx.world)
                        .translation;

                    let player_rotation = ctx
                        .world
                        .query_filtered::<&Transform, With<PlayerCamera>>()
                        .single(ctx.world)
                        .rotation
                        .to_euler(EulerRot::XYZ);

                    (chunks, player_pos, player_rotation)
                })
                .await;

            for (chunk_pos, chunk) in chunks_to_save {
                let data = bincode::serialize(&chunk).unwrap();

                sqlx::query!(
                    "UPDATE chunks SET data = ? WHERE x = ? AND y = ? AND z = ?",
                    data,
                    chunk_pos.x,
                    chunk_pos.y,
                    chunk_pos.z
                )
                .execute(&db)
                .await
                .unwrap();
            }

            sqlx::query!(
                "UPDATE player SET x = ?, y = ?, z = ?, roll = ?, pitch = ?, yaw = ?",
                player_pos.x,
                player_pos.y,
                player_pos.z,
                player_rotation.0,
                player_rotation.1,
                player_rotation.2
            )
            .execute(&db)
            .await
            .unwrap();

            sleep(Duration::from_millis(250)).await;
        }
    });
}

fn build_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    level: Res<Level>,
    dirty_query: Query<(Entity, &ChunkPos), With<Dirty>>,
) {
    let items = dirty_query.iter().collect::<Vec<_>>();

    let items = items
        .into_par_iter()
        .map(|(entity, &chunk_pos)| {
            let chunk = level.chunk(chunk_pos)?;
            let mesh = chunk.render(&level, chunk_pos).build();
            Some((entity, mesh))
        })
        .collect::<Vec<_>>();

    for item in items {
        let Some((entity, mesh)) = item else {
            continue;
        };

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(mesh)))
            .remove::<Dirty>();
    }
}

fn unload_distant_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    player_query: Query<&Transform, With<Player>>,
    runtime: Res<TokioTasksRuntime>,
    db: Res<LevelDatabase>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let player_chunk = BlockPos::from_world(player_pos).chunk_pos();
    let unload_distance_sq = CHUNK_UNLOAD_DISTANCE * CHUNK_UNLOAD_DISTANCE;

    // Collect chunks to unload
    let chunks_to_unload: Vec<_> = level
        .chunks
        .iter()
        .filter(|(&chunk_pos, _)| {
            let dx = chunk_pos.x - player_chunk.x;
            let dy = chunk_pos.y - player_chunk.y;
            let dz = chunk_pos.z - player_chunk.z;
            dx * dx + dy * dy + dz * dz > unload_distance_sq
        })
        .map(|(&pos, loaded)| (pos, loaded.entity))
        .collect();

    // Unload chunks
    for (chunk_pos, entity) in chunks_to_unload {
        if let Some(loaded_chunk) = level.chunks.remove(&chunk_pos) {
            // Save chunk data before unloading
            let data = bincode::serialize(&loaded_chunk.chunk).unwrap();
            let db = db.0.clone();

            runtime.spawn_background_task(move |_ctx| async move {
                sqlx::query!(
                    "UPDATE chunks SET data = ? WHERE x = ? AND y = ? AND z = ?",
                    data,
                    chunk_pos.x,
                    chunk_pos.y,
                    chunk_pos.z
                )
                .execute(&db)
                .await
                .unwrap();
            });

            // Despawn the chunk entity
            commands.entity(entity).despawn();
        }
    }
}
