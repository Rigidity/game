#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Twig,
    PlantFiber,
    Flint,
}

impl Item {
    pub fn get_texture_path(&self) -> &'static str {
        match self {
            Item::Twig => "Items/Twig.png",
            Item::PlantFiber => "Items/PlantFiber.png",
            Item::Flint => "Items/Flint.png",
        }
    }
}
