use bevy::{prelude::*, utils::HashMap, utils::HashSet};
use bevy_tokio_tasks::TokioTasksRuntime;
use noise::{NoiseFn, Perlin};
use sqlx::SqlitePool;

use crate::{
    block::Block,
    chunk::Chunk,
    game_state::GameState,
    loader::GlobalVoxelMaterial,
    player::Player,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_SIZE},
};

const CHUNK_UNLOAD_DISTANCE: i32 = 16; // Should be larger than generation radius

#[derive(Debug, Clone, Copy)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Level::new())
            .insert_resource(LevelGenerator::new(42))
            .insert_resource(ChunkGenerationQueue::default())
            .insert_resource(ChunkGenerationTask::default())
            .add_systems(OnEnter(GameState::Setup), setup_level)
            .add_systems(
                Update,
                (
                    generate_chunk_batch,
                    build_chunk_meshes,
                    unload_distant_chunks,
                )
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
pub struct LevelGenerator {
    noise: Perlin,
}

impl LevelGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    fn get_density_factor(&self, pos: &Vec3) -> f64 {
        let large_scale = self.noise.get([
            pos.x as f64 * 0.005,
            pos.y as f64 * 0.005,
            pos.z as f64 * 0.005,
        ]);

        large_scale * 0.6
    }

    fn get_terrain_density(&self, pos: &Vec3) -> f64 {
        self.noise.get([
            pos.x as f64 * 0.02,
            pos.y as f64 * 0.02,
            pos.z as f64 * 0.02,
        ])
    }

    fn generate_chunk(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = LocalPos::new(x, y, z);
                    let pos = local_pos.block_pos(chunk_pos).world_pos();

                    let density_factor = self.get_density_factor(&pos);
                    let terrain_density = self.get_terrain_density(&pos);

                    if terrain_density > density_factor {
                        // Check upwards until we find air to determine if we're near a surface
                        let distance_to_surface = (0..=5)
                            .find(|&d| {
                                let check_pos =
                                    (local_pos.block_pos(chunk_pos) + BlockPos::Y * d).world_pos();
                                let check_density_factor = self.get_density_factor(&check_pos);
                                let check_terrain_density = self.get_terrain_density(&check_pos);
                                check_terrain_density <= check_density_factor
                            })
                            .unwrap_or(5);

                        let block_type = if distance_to_surface == 1 {
                            Block::Grass
                        } else if distance_to_surface <= 3 {
                            Block::Dirt
                        } else {
                            Block::Rock
                        };

                        chunk.set(local_pos, block_type);
                    }
                }
            }
        }

        chunk
    }
}

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

#[derive(Debug, Default, Resource)]
struct ChunkGenerationTask {
    running: bool,
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

fn generate_chunk_batch(
    material: Res<GlobalVoxelMaterial>,
    db: Res<LevelDatabase>,
    runtime: Res<TokioTasksRuntime>,
    generator: Res<LevelGenerator>,
    mut task_status: ResMut<ChunkGenerationTask>,
) {
    if task_status.running {
        return;
    }

    task_status.running = true;
    let material = material.0.clone();
    let db = db.0.clone();
    let mut generator = generator.clone();

    runtime.spawn_background_task(|mut ctx| async move {
        loop {
            // Get the highest priority chunk to generate
            let chunk_pos = ctx
                .run_on_main_thread(|ctx| {
                    let mut player_query = ctx.world.query::<&Transform>();

                    let level = ctx.world.resource::<Level>();
                    let queue = ctx.world.resource::<ChunkGenerationQueue>();

                    // Get player position
                    let player_transform = player_query.iter(ctx.world).next()?;
                    let player_pos = player_transform.translation;
                    let player_chunk = BlockPos::from_world(player_pos).chunk_pos();

                    // Find closest non-generated chunk within radius
                    let radius = 12;
                    let mut best_chunk = None;
                    let mut best_priority = None;

                    for x in -radius..=radius {
                        for y in -radius..=radius {
                            for z in -radius..=radius {
                                let pos = ChunkPos::new(
                                    player_chunk.x + x,
                                    player_chunk.y + y,
                                    player_chunk.z + z,
                                );

                                let distance_sq = x * x + y * y + z * z;
                                if distance_sq > radius * radius {
                                    continue;
                                }

                                if level.chunk(pos).is_some() || queue.pending.contains(&pos) {
                                    continue;
                                }

                                let dy = y;
                                let below = dy <= 0;
                                let horizontal_dist = x * x + z * z;

                                let priority = (!below, horizontal_dist, dy.abs(), distance_sq);

                                if best_priority.is_none_or(|p| priority < p) {
                                    best_priority = Some(priority);
                                    best_chunk = Some(pos);
                                }
                            }
                        }
                    }

                    best_chunk
                })
                .await;

            let Some(chunk_pos) = chunk_pos else {
                // No chunks need generation, sleep briefly and check again
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            };

            // Mark chunk as pending
            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .resource_mut::<ChunkGenerationQueue>()
                    .pending
                    .insert(chunk_pos);
            })
            .await;

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

            let material = material.clone();

            // Add the chunk to the world
            ctx.run_on_main_thread(move |ctx| {
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

                ctx.world
                    .resource_mut::<Level>()
                    .chunks
                    .insert(chunk_pos, LoadedChunk { chunk, entity });

                ctx.world
                    .resource_mut::<ChunkGenerationQueue>()
                    .pending
                    .remove(&chunk_pos);
            })
            .await;
        }
    });
}

fn build_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    runtime: Res<TokioTasksRuntime>,
    db: Res<LevelDatabase>,
    level: Res<Level>,
    dirty_query: Query<(Entity, &ChunkPos), With<Dirty>>,
    modified_query: Query<(Entity, &ChunkPos), With<Modified>>,
) {
    for (entity, &chunk_pos) in dirty_query.iter().chain(modified_query.iter()) {
        let Some(chunk) = level.chunk(chunk_pos) else {
            continue;
        };

        let mesh = chunk.render(&level, chunk_pos).build();

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(mesh)))
            .remove::<Dirty>()
            .remove::<Modified>();

        let data = bincode::serialize(&chunk).unwrap();
        let db = db.0.clone();

        if modified_query.contains(entity) {
            runtime.spawn_background_task(move |mut _ctx| async move {
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
        }
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
