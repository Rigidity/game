use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub count: u32,
}

impl Item {
    pub fn new(kind: ItemKind, count: u32) -> Self {
        Self { kind, count }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Twig,
    PlantFiber,
    Flint,
    Soil,
    Handle(ToolPart),
    Binding(ToolPart),
    PickaxeHead(ToolPart),
    Pickaxe {
        handle: ToolPart,
        binding: ToolPart,
        head: ToolPart,
    },
}

impl ItemKind {
    pub fn is_stackable(&self) -> bool {
        match self {
            Self::Twig | Self::PlantFiber | Self::Flint | Self::Soil => true,
            Self::Handle(..) | Self::Binding(..) | Self::PickaxeHead(..) | Self::Pickaxe { .. } => {
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ToolPart {
    pub material: Material,
    pub durability: u32,
}

impl ToolPart {
    pub fn new(material: Material) -> Self {
        Self {
            material,
            durability: material.durability(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Material {
    Twig,
    PlantFiber,
    Flint,
}

impl Material {
    pub fn durability(&self) -> u32 {
        match self {
            Self::Twig => 100,
            Self::PlantFiber => 100,
            Self::Flint => 200,
        }
    }

    pub fn hardness(&self) -> f32 {
        match self {
            Self::Twig => 1.0,
            Self::PlantFiber => 0.5,
            Self::Flint => 2.0,
        }
    }
}

impl fmt::Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Twig => "Twig",
                Self::PlantFiber => "Plant Fiber",
                Self::Flint => "Flint",
            }
        )
    }
}

impl fmt::Display for ToolPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            if self.durability == 0 {
                format!("Broken {}", self.material)
            } else {
                format!("{}", self.material)
            }
        )
    }
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Twig => "Twig".to_string(),
                Self::PlantFiber => "Plant Fiber".to_string(),
                Self::Flint => "Flint".to_string(),
                Self::Soil => "Soil".to_string(),
                Self::Handle(part) => format!("{part} Handle"),
                Self::Binding(part) => format!("{part} Binding"),
                Self::PickaxeHead(part) => format!("{part} Pickaxe Head"),
                Self::Pickaxe { head, .. } => format!("{head} Pickaxe"),
            }
        )
    }
}
