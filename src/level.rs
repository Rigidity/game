use bevy::{prelude::*, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::Block,
    chunk::Chunk,
    game_state::GameState,
    loader::GlobalVoxelMaterial,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_SIZE},
};

#[derive(Debug, Clone, Copy)]
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Level::new(42))
            .add_systems(OnEnter(GameState::Playing), generate_chunks)
            .add_systems(
                Update,
                build_chunk_meshes.run_if(in_state(GameState::Playing)),
            );
    }
}

struct LoadedChunk {
    chunk: Chunk,
    entity: Entity,
}

#[derive(Resource)]
pub struct Level {
    chunks: HashMap<ChunkPos, LoadedChunk>,
    noise: Perlin,
}

impl Level {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            noise: Perlin::new(seed),
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

    fn generate_chunk(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = LocalPos::new(x, y, z);
                    let pos = local_pos.block_pos(chunk_pos).world_pos();

                    let height = self.noise.get([pos.x as f64 * 0.02, pos.z as f64 * 0.02]);
                    let surface_height = height * 18.0 + 60.0;

                    if (pos.y as f64) < surface_height {
                        let block_type = if (surface_height - (pos.y as f64)) <= 1.0 {
                            Block::Grass
                        } else if (surface_height - (pos.y as f64)) <= 3.0 {
                            Block::Dirt
                        } else {
                            Block::Rock
                        };

                        chunk.set(LocalPos::new(x, y, z), block_type);
                    }
                }
            }
        }

        chunk
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Dirty;

fn generate_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    material: Res<GlobalVoxelMaterial>,
) {
    for chunk_x in 0..16 {
        for chunk_y in 0..16 {
            for chunk_z in 0..16 {
                let chunk_pos = ChunkPos::new(chunk_x, chunk_y, chunk_z);
                let chunk = level.generate_chunk(chunk_pos);

                let entity = commands
                    .spawn((
                        chunk_pos,
                        Dirty,
                        MeshMaterial3d(material.0.clone()),
                        Transform::from_xyz(
                            chunk_pos.x as f32 * 16.0,
                            chunk_pos.y as f32 * 16.0,
                            chunk_pos.z as f32 * 16.0,
                        ),
                    ))
                    .id();

                level
                    .chunks
                    .insert(chunk_pos, LoadedChunk { chunk, entity });
            }
        }
    }
}

fn build_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    level: Res<Level>,
    query: Query<(Entity, &ChunkPos), With<Dirty>>,
) {
    for (entity, &chunk_pos) in query.iter() {
        let Some(chunk) = level.chunk(chunk_pos) else {
            continue;
        };

        let mesh = chunk.render(&level, chunk_pos).build();

        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(mesh)))
            .remove::<Dirty>();
    }
}
