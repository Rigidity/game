use std::hash::{Hash, Hasher};

use bevy::{prelude::*, utils::HashMap, utils::HashSet};
use bevy_tokio_tasks::TokioTasksRuntime;
use noise::{NoiseFn, Perlin, Seedable};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
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
const CHUNK_GENERATION_BATCH_SIZE: usize = 8; // Adjust this value as needed
const TREE_HEIGHT: i32 = 12; // Tall but not gigantic
const TREE_RADIUS: i32 = 3; // Reasonable canopy size
const HOUSE_SIZE: i32 = 5;
const STRUCTURE_ATTEMPT_SPACING: i32 = 10; // Increased structure density

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

        large_scale * 0.5
    }

    fn get_terrain_density(&self, pos: &Vec3) -> f64 {
        self.noise.get([
            pos.x as f64 * 0.02,
            pos.y as f64 * 0.02,
            pos.z as f64 * 0.02,
        ])
    }

    fn get_structure_rng(&self, pos: BlockPos) -> ChaCha8Rng {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        pos.hash(&mut hasher);
        let hash = hasher.finish();
        ChaCha8Rng::seed_from_u64(hash ^ self.noise.seed() as u64)
    }

    // Add this helper function to find structure origins that could affect a chunk
    fn get_structure_positions_affecting_chunk(&self, chunk_pos: ChunkPos) -> Vec<BlockPos> {
        let chunk_min = chunk_pos.world_pos();
        let chunk_max = (chunk_pos + ChunkPos::new(1, 1, 1)).world_pos();
        let structure_radius = (HOUSE_SIZE.max(TREE_HEIGHT + TREE_RADIUS) + 1) as f32;

        // Calculate the grid positions that could affect this chunk
        let min_x =
            ((chunk_min.x - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_x =
            ((chunk_max.x + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;
        let min_z =
            ((chunk_min.z - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_z =
            ((chunk_max.z + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;

        // Calculate Y range to check for structures that could affect this chunk
        let check_height = TREE_HEIGHT + TREE_RADIUS;
        let min_y = chunk_pos.y - (check_height / CHUNK_SIZE as i32) - 1;
        let max_y = chunk_pos.y + (check_height / CHUNK_SIZE as i32) + 1;

        let mut positions = Vec::new();

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    // Get the structure origin point
                    let origin = BlockPos::new(
                        x * STRUCTURE_ATTEMPT_SPACING,
                        y * CHUNK_SIZE as i32,
                        z * STRUCTURE_ATTEMPT_SPACING,
                    );
                    positions.push(origin);
                }
            }
        }

        positions
    }

    fn set_block_world(&self, chunks: &mut HashMap<ChunkPos, Chunk>, pos: BlockPos, block: Block) {
        let chunk_pos = pos.chunk_pos();
        let local_pos = pos.local_pos();

        // Get or create the chunk
        chunks.entry(chunk_pos).or_default().set(local_pos, block);
    }

    fn generate_tree(
        &self,
        chunks: &mut HashMap<ChunkPos, Chunk>,
        chunk_pos: ChunkPos,
        base_pos: LocalPos,
        rng: &mut ChaCha8Rng,
    ) {
        let base_world_pos = base_pos.block_pos(chunk_pos);
        let height = TREE_HEIGHT + (rng.random::<i32>() % 3); // Less variation

        // Generate single-block trunk
        for y in 0..height {
            let pos = base_world_pos + BlockPos::new(0, y, 0);
            self.set_block_world(chunks, pos, Block::Dirt); // Dirt trunk
        }

        // Generate Minecraft-style leaf arrangement
        let leaf_start = height - 4;
        let leaf_height = 5;

        for y in 0..leaf_height {
            let y_pos = leaf_start + y;
            // Radius varies by height - wider in middle, narrower at top/bottom
            let radius = if y == 0 || y == leaf_height - 1 { 1 } else { 2 };

            for x in -radius..=radius {
                for z in -radius..=radius {
                    // Skip corners for rounder appearance
                    if x * x + z * z > radius * radius + 1 {
                        continue;
                    }

                    // Add some randomness to leaf placement
                    if (x == radius || x == -radius || z == radius || z == -radius)
                        && rng.random::<f32>() < 0.5
                    {
                        continue;
                    }

                    let pos = base_world_pos + BlockPos::new(x, y_pos, z);
                    self.set_block_world(chunks, pos, Block::Leaves);
                }
            }
        }

        // Add a few random extra leaves for variety
        for _ in 0..3 {
            let x = rng.random_range(-2..=2);
            let z = rng.random_range(-2..=2);
            let y = rng.random_range(leaf_start..leaf_start + leaf_height);

            if x != 0 || z != 0 {
                // Don't place on trunk
                let pos = base_world_pos + BlockPos::new(x, y, z);
                self.set_block_world(chunks, pos, Block::Leaves);
            }
        }
    }

    fn generate_house(
        &self,
        chunks: &mut HashMap<ChunkPos, Chunk>,
        chunk_pos: ChunkPos,
        base_pos: LocalPos,
        rng: &mut ChaCha8Rng,
    ) {
        let base_world_pos = base_pos.block_pos(chunk_pos);
        let size = HOUSE_SIZE + (rng.random::<i32>() % 2);
        let height = size - 1;

        // Generate walls
        for y in 0..height {
            for x in 0..size {
                for z in 0..size {
                    if x == 0 || x == size - 1 || z == 0 || z == size - 1 {
                        let pos = base_world_pos + BlockPos::new(x, y, z);
                        self.set_block_world(chunks, pos, Block::Rock);
                    }
                }
            }
        }

        // Generate roof
        for x in -1..=size {
            for z in -1..=size {
                let pos = base_world_pos + BlockPos::new(x, height, z);
                self.set_block_world(chunks, pos, Block::Rock);
            }
        }

        // Add door
        let door_pos = base_world_pos + BlockPos::new(0, 1, size / 2);
        self.set_block_world(chunks, door_pos, Block::Air);
    }

    fn generate_chunk(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunks = HashMap::new();
        let mut chunk = Chunk::new();

        // Generate terrain first
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

        // Insert the terrain-generated chunk into the chunks map
        chunks.insert(chunk_pos, chunk);

        // Get all possible structure positions that could affect this chunk
        let structure_positions = self.get_structure_positions_affecting_chunk(chunk_pos);

        // Try generating structures at each position
        for pos in structure_positions {
            let world_pos = pos.world_pos();

            // Check density at structure position
            let density_factor = self.get_density_factor(&world_pos);
            let terrain_density = self.get_terrain_density(&world_pos);

            // Only generate if we're at an air/ground boundary
            if terrain_density > density_factor {
                continue;
            }

            let below_pos = world_pos + Vec3::new(0.0, -1.0, 0.0);
            let below_density_factor = self.get_density_factor(&below_pos);
            let below_terrain_density = self.get_terrain_density(&below_pos);
            if below_terrain_density <= below_density_factor {
                continue;
            }

            // Use deterministic RNG for this position
            let mut rng = self.get_structure_rng(pos);

            // Generate the structure
            if rng.random::<f32>() < 0.9 {
                self.generate_tree(&mut chunks, pos.chunk_pos(), pos.local_pos(), &mut rng);
            } else {
                self.generate_house(&mut chunks, pos.chunk_pos(), pos.local_pos(), &mut rng);
            }
        }

        // Return the current chunk
        chunks.remove(&chunk_pos).unwrap_or_else(Chunk::new)
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
            let chunk_positions = ctx
                .run_on_main_thread(|ctx| {
                    let mut player_query = ctx.world.query_filtered::<&Transform, With<Player>>();
                    let level = ctx.world.resource::<Level>();
                    let queue = ctx.world.resource::<ChunkGenerationQueue>();

                    // Get player position - if no player, stop generating
                    let player_transform = match player_query.iter(ctx.world).next() {
                        Some(transform) => transform,
                        None => {
                            ctx.world.resource_mut::<ChunkGenerationTask>().running = false;
                            return Vec::new();
                        }
                    };

                    let player_pos = player_transform.translation;
                    let player_chunk = BlockPos::from_world(player_pos).chunk_pos();

                    // Find closest non-generated chunks within radius
                    let radius = 12;
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
                // No chunks need generation, sleep briefly and check again
                // Only stop the task if there are no pending chunks
                if ctx
                    .run_on_main_thread(|ctx| {
                        if ctx
                            .world
                            .resource::<ChunkGenerationQueue>()
                            .pending
                            .is_empty()
                        {
                            ctx.world.resource_mut::<ChunkGenerationTask>().running = false;
                            true
                        } else {
                            false
                        }
                    })
                    .await
                {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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

            let material = material.clone();

            // Add the chunks to the world
            ctx.run_on_main_thread(move |ctx| {
                for (chunk_pos, chunk) in chunks {
                    let entity = ctx
                        .world
                        .spawn((
                            chunk_pos,
                            Dirty,
                            MeshMaterial3d(material.clone()),
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
                }
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
