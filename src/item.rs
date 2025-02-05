use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Item {
    Twig,
    PlantFiber,
    Flint,
    Soil,
    Handle(HandleMaterial),
    Binding(BindingMaterial),
    HatchetHead(HeadMaterial),
    Hatchet {
        handle: HandleMaterial,
        binding: BindingMaterial,
        head: HeadMaterial,
    },
}

impl Item {
    pub fn get_texture_path(&self) -> &'static str {
        match self {
            Item::Twig => "Items/Items - Twig.png",
            Item::PlantFiber => "Items/Items - Plant Fiber.png",
            Item::Flint => "Items/Items - Flint.png",
            Item::Soil => "Items/Items - Soil.png",
            Item::Handle(..) => "Items/Items - Handle.png",
            Item::Binding(..) => "Items/Items - Binding.png",
            Item::HatchetHead(..) => "Items/Items - Hatchet Head.png",
            Item::Hatchet {
                handle,
                binding,
                head,
            } => match (handle, binding, head) {
                (HandleMaterial::Wood, BindingMaterial::PlantFiber, HeadMaterial::Flint) => {
                    "Items/Items - Hatchet.png"
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HandleMaterial {
    Wood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BindingMaterial {
    PlantFiber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HeadMaterial {
    Flint,
}
