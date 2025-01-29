use bevy::{prelude::*, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::{Block, BlockFaces},
    chunk::Chunk,
    game_state::GameState,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_SIZE},
    texture_array::create_texture_array,
    voxel_material::VoxelMaterial,
    voxel_mesh::VoxelFace,
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

    pub fn ambient_occlusion(&self, block_pos: BlockPos, face: VoxelFace) -> [u32; 4] {
        match face {
            VoxelFace::Top => {
                let top = block_pos.top();

                let [s1, s2, s3, s4] = [
                    self.block(top.back()).is_solid(),
                    self.block(top.right()).is_solid(),
                    self.block(top.front()).is_solid(),
                    self.block(top.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(top.back().left()).is_solid(),
                    self.block(top.back().right()).is_solid(),
                    self.block(top.front().right()).is_solid(),
                    self.block(top.front().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // front-left
                    calculate_corner_ao(s2, s3, c3), // front-right
                    calculate_corner_ao(s2, s1, c2), // back-right
                    calculate_corner_ao(s4, s1, c1), // back-left
                ]
            }
            VoxelFace::Bottom => {
                let bottom = block_pos.bottom();

                let [s1, s2, s3, s4] = [
                    self.block(bottom.back()).is_solid(),
                    self.block(bottom.right()).is_solid(),
                    self.block(bottom.front()).is_solid(),
                    self.block(bottom.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(bottom.back().left()).is_solid(),
                    self.block(bottom.back().right()).is_solid(),
                    self.block(bottom.front().right()).is_solid(),
                    self.block(bottom.front().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // front-left
                    calculate_corner_ao(s2, s3, c3), // front-right
                    calculate_corner_ao(s2, s1, c2), // back-right
                    calculate_corner_ao(s4, s1, c1), // back-left
                ]
            }
            VoxelFace::Left => {
                let left = block_pos.left();

                let [s1, s2, s3, s4] = [
                    self.block(left.back()).is_solid(),
                    self.block(left.top()).is_solid(),
                    self.block(left.front()).is_solid(),
                    self.block(left.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(left.bottom().back()).is_solid(),
                    self.block(left.top().back()).is_solid(),
                    self.block(left.top().front()).is_solid(),
                    self.block(left.bottom().front()).is_solid(),
                ];
                [
                    calculate_corner_ao(s2, s1, c2), // top-back
                    calculate_corner_ao(s4, s1, c1), // bottom-back
                    calculate_corner_ao(s4, s3, c4), // bottom-front
                    calculate_corner_ao(s2, s3, c3), // top-front
                ]
            }
            VoxelFace::Right => {
                let right = block_pos.right();

                let [s1, s2, s3, s4] = [
                    self.block(right.back()).is_solid(),
                    self.block(right.top()).is_solid(),
                    self.block(right.front()).is_solid(),
                    self.block(right.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(right.bottom().back()).is_solid(),
                    self.block(right.top().back()).is_solid(),
                    self.block(right.top().front()).is_solid(),
                    self.block(right.bottom().front()).is_solid(),
                ];

                [
                    calculate_corner_ao(s2, s1, c2), // top-back (TopLeft)
                    calculate_corner_ao(s4, s1, c1), // bottom-back (BottomLeft)
                    calculate_corner_ao(s4, s3, c4), // bottom-front (BottomRight)
                    calculate_corner_ao(s2, s3, c3), // top-front (TopRight)
                ]
            }
            VoxelFace::Front => {
                let front = block_pos.front();

                let [s1, s2, s3, s4] = [
                    self.block(front.bottom()).is_solid(),
                    self.block(front.right()).is_solid(),
                    self.block(front.top()).is_solid(),
                    self.block(front.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(front.bottom().left()).is_solid(),
                    self.block(front.bottom().right()).is_solid(),
                    self.block(front.top().right()).is_solid(),
                    self.block(front.top().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // top-left
                    calculate_corner_ao(s4, s1, c1), // bottom-left
                    calculate_corner_ao(s2, s1, c2), // bottom-right
                    calculate_corner_ao(s2, s3, c3), // top-right
                ]
            }
            VoxelFace::Back => {
                let back = block_pos.back();

                let [s1, s2, s3, s4] = [
                    self.block(back.left()).is_solid(),
                    self.block(back.top()).is_solid(),
                    self.block(back.right()).is_solid(),
                    self.block(back.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    self.block(back.bottom().left()).is_solid(),
                    self.block(back.bottom().right()).is_solid(),
                    self.block(back.top().right()).is_solid(),
                    self.block(back.top().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s2, s1, c4), // top-left
                    calculate_corner_ao(s4, s1, c1), // bottom-left
                    calculate_corner_ao(s4, s3, c2), // bottom-right
                    calculate_corner_ao(s2, s3, c3), // top-right
                ]
            }
        }
    }
}

fn calculate_corner_ao(side1: bool, side2: bool, corner: bool) -> u32 {
    match (side1, side2, corner) {
        (true, true, _) => 0,
        (true, false, false) | (false, true, false) => 1,
        (false, false, true) => 1,
        (false, false, false) => 2,
        _ => 1,
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

                            let noise_value = world.noise.get([
                                world_x as f64 * 0.04,
                                world_y as f64 * 0.04,
                                world_z as f64 * 0.04,
                            ]);

                            let normalized_noise = (noise_value + 1.0) / 2.0;
                            let height_threshold = (world_y as f64 / (4.0 * 16.0)) * 0.8;

                            if normalized_noise > height_threshold {
                                chunk.set(
                                    LocalPos::new(x as usize, y as usize, z as usize).unwrap(),
                                    Block::Rock,
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
    let array_texture = create_texture_array(vec![image_assets.rock.clone()], &mut images).unwrap();
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
