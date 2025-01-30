use bevy::{prelude::*, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::{Block, BlockFaces},
    chunk::Chunk,
    game_state::GameState,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_SIZE},
    texture_array::create_texture_array,
    voxel_material::VoxelMaterial,
    ImageAssets, VoxelMaterials,
};

#[derive(Debug, Clone, Copy)]
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (generate_chunks, build_chunk_meshes).chain(),
        );
    }
}

#[derive(Resource)]
pub struct WorldMap {
    chunks: HashMap<ChunkPos, Chunk>,
    entities: HashMap<ChunkPos, Entity>,
    noise: Perlin,
}

impl WorldMap {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            entities: HashMap::new(),
            noise: Perlin::new(seed),
        }
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn block(&self, pos: BlockPos) -> Block {
        let chunk_pos = pos.chunk_pos();
        let local_pos = pos.local_pos();
        self.chunk(chunk_pos)
            .map(|chunk| chunk.get(local_pos))
            .unwrap_or(Block::Air)
    }

    pub fn visible_faces(&self, pos: BlockPos) -> BlockFaces {
        BlockFaces {
            left: self.block(pos.left()).is_air(),
            right: self.block(pos.right()).is_air(),
            front: self.block(pos.front()).is_air(),
            back: self.block(pos.back()).is_air(),
            top: self.block(pos.top()).is_air(),
            bottom: self.block(pos.bottom()).is_air(),
        }
    }
}

fn generate_chunks(mut world: ResMut<WorldMap>) {
    // Generate a grid of chunks
    for chunk_x in 0..16 {
        for chunk_y in 0..16 {
            for chunk_z in 0..16 {
                let mut chunk = Chunk::new();
                let chunk_pos = ChunkPos::new(chunk_x, chunk_y, chunk_z);

                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        for z in 0..CHUNK_SIZE {
                            let world_x = chunk_x * CHUNK_SIZE + x;
                            let world_y = chunk_y * CHUNK_SIZE + y;
                            let world_z = chunk_z * CHUNK_SIZE + z;

                            let height = world
                                .noise
                                .get([world_x as f64 * 0.02, world_z as f64 * 0.02]);
                            let surface_height = height * 18.0 + 60.0;

                            if (world_y as f64) < surface_height {
                                let block_type = if (surface_height - (world_y as f64)) <= 1.0 {
                                    Block::Grass
                                } else if (surface_height - (world_y as f64)) <= 3.0 {
                                    Block::Dirt
                                } else {
                                    Block::Rock
                                };

                                chunk.set(
                                    LocalPos::new(x as usize, y as usize, z as usize).unwrap(),
                                    block_type,
                                );
                            }
                        }
                    }
                }

                world.chunks.insert(chunk_pos, chunk);
            }
        }
    }
}

fn build_chunk_meshes(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut world: ResMut<WorldMap>,
) {
    let array_texture = create_texture_array(
        vec![
            image_assets.rock.clone(),
            image_assets.dirt.clone(),
            image_assets.grass_side.clone(),
            image_assets.grass.clone(),
        ],
        &mut images,
    )
    .unwrap();
    let material = materials.add(VoxelMaterial { array_texture });

    commands.insert_resource(VoxelMaterials {
        material: material.clone(),
    });

    let mut entities = HashMap::new();

    for (&chunk_pos, chunk) in &world.chunks {
        let mesh = chunk.render(&world, chunk_pos).build();

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(
                    chunk_pos.x as f32 * 16.0,
                    chunk_pos.y as f32 * 16.0,
                    chunk_pos.z as f32 * 16.0,
                ),
            ))
            .id();

        entities.insert(chunk_pos, entity);
    }

    world.entities.extend(entities);
}

pub fn regenerate_chunk_mesh(
    commands: &mut Commands,
    world: &mut WorldMap,
    chunk_pos: ChunkPos,
    meshes: &mut Assets<Mesh>,
    materials: &VoxelMaterials,
) {
    let mesh = world
        .chunk(chunk_pos)
        .unwrap()
        .render(world, chunk_pos)
        .build();

    let new_id = commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.material.clone()),
            Transform::from_xyz(
                chunk_pos.x as f32 * 16.0,
                chunk_pos.y as f32 * 16.0,
                chunk_pos.z as f32 * 16.0,
            ),
        ))
        .id();

    if let Some(entity) = world.entities.insert(chunk_pos, new_id) {
        commands.entity(entity).despawn();
    }
}
