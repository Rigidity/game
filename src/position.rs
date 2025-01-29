use std::ops::Add;

use bevy::prelude::*;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_INDICES: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

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
            self.x.div_euclid(CHUNK_SIZE),
            self.y.div_euclid(CHUNK_SIZE),
            self.z.div_euclid(CHUNK_SIZE),
        )
    }

    pub fn local_pos(self) -> LocalPos {
        LocalPos {
            x: self.x.rem_euclid(CHUNK_SIZE),
            y: self.y.rem_euclid(CHUNK_SIZE),
            z: self.z.rem_euclid(CHUNK_SIZE),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    x: i32,
    y: i32,
    z: i32,
}

impl LocalPos {
    pub fn new(x: usize, y: usize, z: usize) -> Option<Self> {
        if x >= CHUNK_SIZE as usize || y >= CHUNK_SIZE as usize || z >= CHUNK_SIZE as usize {
            return None;
        }
        Some(Self {
            x: x as i32,
            y: y as i32,
            z: z as i32,
        })
    }

    pub fn from_index(index: usize) -> Self {
        let index = index as i32;
        let x = index % CHUNK_SIZE;
        let y = (index / CHUNK_SIZE) % CHUNK_SIZE;
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        Self { x, y, z }
    }

    pub fn x(&self) -> usize {
        self.x as usize
    }

    pub fn y(&self) -> usize {
        self.y as usize
    }

    pub fn z(&self) -> usize {
        self.z as usize
    }

    pub fn index(self) -> usize {
        (self.x + self.y * CHUNK_SIZE + self.z * CHUNK_SIZE * CHUNK_SIZE) as usize
    }

    pub fn block_pos(self, chunk_pos: ChunkPos) -> BlockPos {
        BlockPos::new(
            chunk_pos.x * CHUNK_SIZE + self.x,
            chunk_pos.y * CHUNK_SIZE + self.y,
            chunk_pos.z * CHUNK_SIZE + self.z,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            (self.x * CHUNK_SIZE) as f32,
            (self.y * CHUNK_SIZE) as f32,
            (self.z * CHUNK_SIZE) as f32,
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
