use std::ops::{Add, Mul};

use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_INDICES: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const X: Self = Self { x: 1, y: 0, z: 0 };
    pub const Y: Self = Self { x: 0, y: 1, z: 0 };
    pub const Z: Self = Self { x: 0, y: 0, z: 1 };
    pub const NEG_X: Self = Self { x: -1, y: 0, z: 0 };
    pub const NEG_Y: Self = Self { x: 0, y: -1, z: 0 };
    pub const NEG_Z: Self = Self { x: 0, y: 0, z: -1 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(world_pos: Vec3) -> Self {
        Self {
            x: world_pos.x.floor() as i32,
            y: world_pos.y.floor() as i32,
            z: world_pos.z.floor() as i32,
        }
    }

    pub fn chunk_pos(self) -> ChunkPos {
        ChunkPos::new(
            self.x.div_euclid(CHUNK_SIZE as i32),
            self.y.div_euclid(CHUNK_SIZE as i32),
            self.z.div_euclid(CHUNK_SIZE as i32),
        )
    }

    pub fn local_pos(self) -> LocalPos {
        LocalPos {
            x: self.x.rem_euclid(CHUNK_SIZE as i32) as usize,
            y: self.y.rem_euclid(CHUNK_SIZE as i32) as usize,
            z: self.z.rem_euclid(CHUNK_SIZE as i32) as usize,
        }
    }

    pub fn world_pos(self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn center(self) -> Vec3 {
        Vec3::new(
            self.x as f32 + 0.5,
            self.y as f32 + 0.5,
            self.z as f32 + 0.5,
        )
    }

    pub fn left(self) -> Self {
        self + Self::NEG_X
    }

    pub fn right(self) -> Self {
        self + Self::X
    }

    pub fn front(self) -> Self {
        self + Self::Z
    }

    pub fn back(self) -> Self {
        self + Self::NEG_Z
    }

    pub fn top(self) -> Self {
        self + Self::Y
    }

    pub fn bottom(self) -> Self {
        self + Self::NEG_Y
    }
}

impl Add for BlockPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<i32> for BlockPos {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl LocalPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn from_index(index: usize) -> Self {
        let x = index % CHUNK_SIZE;
        let y = (index / CHUNK_SIZE) % CHUNK_SIZE;
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        Self { x, y, z }
    }

    pub fn index(self) -> usize {
        self.x + self.y * CHUNK_SIZE + self.z * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn block_pos(self, chunk_pos: ChunkPos) -> BlockPos {
        BlockPos::new(
            chunk_pos.x * CHUNK_SIZE as i32 + self.x as i32,
            chunk_pos.y * CHUNK_SIZE as i32 + self.y as i32,
            chunk_pos.z * CHUNK_SIZE as i32 + self.z as i32,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const X: Self = Self { x: 1, y: 0, z: 0 };
    pub const Y: Self = Self { x: 0, y: 1, z: 0 };
    pub const Z: Self = Self { x: 0, y: 0, z: 1 };
    pub const NEG_X: Self = Self { x: -1, y: 0, z: 0 };
    pub const NEG_Y: Self = Self { x: 0, y: -1, z: 0 };
    pub const NEG_Z: Self = Self { x: 0, y: 0, z: -1 };

    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn world_pos(self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE as f32,
            self.y as f32 * CHUNK_SIZE as f32,
            self.z as f32 * CHUNK_SIZE as f32,
        )
    }
}

impl Add for ChunkPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
