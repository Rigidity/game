use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Item {
    Twig,
    PlantFiber,
    Flint,
    Soil,
    Handle(HandleMaterial),
    Binding(BindingMaterial),
    PickaxeHead(HeadMaterial),
    Pickaxe {
        handle: HandleMaterial,
        binding: BindingMaterial,
        head: HeadMaterial,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Material {
    Twig,
    PlantFiber,
    Flint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HandleMaterial {
    Twig,
}

impl From<HandleMaterial> for Material {
    fn from(material: HandleMaterial) -> Self {
        match material {
            HandleMaterial::Twig => Material::Twig,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BindingMaterial {
    PlantFiber,
}

impl From<BindingMaterial> for Material {
    fn from(material: BindingMaterial) -> Self {
        match material {
            BindingMaterial::PlantFiber => Material::PlantFiber,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HeadMaterial {
    Flint,
}

impl From<HeadMaterial> for Material {
    fn from(material: HeadMaterial) -> Self {
        match material {
            HeadMaterial::Flint => Material::Flint,
        }
    }
}
