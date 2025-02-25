use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::item::{Item, ItemKind, Material, ToolPart};
use crate::level::Level;
use crate::position::BlockPos;
use crate::voxel_mesh::{VoxelFace, VoxelMesh};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BlockFaces {
    pub left: bool,
    pub right: bool,
    pub front: bool,
    pub back: bool,
    pub top: bool,
    pub bottom: bool,
}

impl BlockFaces {
    pub fn get(self, face: VoxelFace) -> bool {
        match face {
            VoxelFace::Left => self.left,
            VoxelFace::Right => self.right,
            VoxelFace::Front => self.front,
            VoxelFace::Back => self.back,
            VoxelFace::Top => self.top,
            VoxelFace::Bottom => self.bottom,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air,
    Rock,
    Dirt,
    Grass,
    Leaves,
    Wood,
    Sand,
    Water,
    Gravel,
}

impl Block {
    pub fn drops(&self) -> Vec<Item> {
        let mut rng = rand::rng();

        match self {
            Self::Leaves => {
                let mut drops = Vec::new();

                if rng.random_bool(0.1) {
                    drops.push(Item::new(ItemKind::Twig, 1));
                }

                if rng.random_bool(0.025) {
                    drops.push(Item::new(ItemKind::PlantFiber, 1));
                }

                drops
            }
            Self::Grass => {
                let mut drops = vec![Item::new(ItemKind::Soil, 1)];

                if rng.random_bool(0.1) {
                    drops.push(Item::new(ItemKind::PlantFiber, 1));
                }

                drops
            }
            Self::Gravel => {
                let mut drops = Vec::new();

                if rng.random_bool(0.1) {
                    drops.push(Item::new(ItemKind::Flint, 1));
                }

                drops
            }
            Self::Dirt => vec![
                Item::new(ItemKind::Soil, 1),
                Item::new(ItemKind::SmallBottle, 1),
            ],
            Self::Sand => vec![
                Item::new(ItemKind::Handle(ToolPart::new(Material::Twig)), 1),
                Item::new(ItemKind::PickaxeHead(ToolPart::new(Material::Flint)), 1),
                Item::new(ItemKind::Binding(ToolPart::new(Material::PlantFiber)), 1),
                Item::new(
                    ItemKind::Pickaxe {
                        handle: ToolPart::new(Material::Twig),
                        binding: ToolPart::new(Material::PlantFiber),
                        head: ToolPart::new(Material::Flint),
                    },
                    1,
                ),
            ],
            _ => Vec::new(),
        }
    }

    pub fn is_breakable_by(self, item: Option<Item>) -> bool {
        match (item, self) {
            (_, Self::Dirt | Self::Grass | Self::Sand | Self::Gravel | Self::Leaves)
            | (
                Some(Item {
                    kind: ItemKind::Pickaxe { .. },
                    ..
                }),
                Self::Rock,
            ) => true,
            (_, Self::Rock | Self::Air | Self::Wood | Self::Water) => false,
        }
    }

    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Leaves | Self::Water)
    }

    pub fn render(
        self,
        mesh: &mut VoxelMesh,
        level: &Level,
        block_pos: BlockPos,
        faces: BlockFaces,
    ) {
        for face in [
            VoxelFace::Left,
            VoxelFace::Right,
            VoxelFace::Front,
            VoxelFace::Back,
            VoxelFace::Top,
            VoxelFace::Bottom,
        ] {
            if faces.get(face) {
                mesh.render_face(
                    level,
                    block_pos,
                    face,
                    #[allow(clippy::match_same_arms)]
                    match (self, face) {
                        (Self::Air, _) => continue,
                        (Self::Rock, _) => 0,
                        (Self::Dirt, _) => 1,
                        (Self::Grass, VoxelFace::Top) => 3,
                        (Self::Grass, VoxelFace::Bottom) => 1,
                        (Self::Grass, _) => 2,
                        (Self::Leaves, _) => 4,
                        (Self::Wood, VoxelFace::Top | VoxelFace::Bottom) => 6,
                        (Self::Wood, _) => 5,
                        (Self::Sand, _) => 7,
                        (Self::Water, _) => 8,
                        (Self::Gravel, _) => 9,
                    },
                );
            }
        }
    }
}
