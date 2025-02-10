use bevy::{prelude::*, utils::HashMap};

use crate::{
    item::{Item, ItemKind, Material},
    loader::ItemImages,
};

#[derive(Debug, Default, Clone, Resource)]
pub struct ItemImageCache {
    images: HashMap<Item, Handle<Image>>,
}

impl ItemImageCache {
    pub fn get(
        &mut self,
        item: Item,
        images: &mut Assets<Image>,
        handles: &ItemImages,
    ) -> Handle<Image> {
        if let Some(image) = self.images.get(&item) {
            return image.clone();
        }

        let (handle, material) = match item.kind {
            ItemKind::Flint => return handles.flint.clone(),
            ItemKind::Soil => return handles.soil.clone(),
            ItemKind::Twig => return handles.twig.clone(),
            ItemKind::PlantFiber => return handles.plant_fiber.clone(),
            ItemKind::Glass => return handles.glass.clone(),
            ItemKind::Handle(part) => (handles.handle.clone(), part.material),
            ItemKind::Binding(part) => (handles.binding.clone(), part.material),
            ItemKind::PickaxeHead(part) => (handles.pickaxe_head.clone(), part.material),
            ItemKind::ShovelHead(part) => (handles.shovel_head.clone(), part.material),
            ItemKind::HatchetHead(part) => (handles.hatchet_head.clone(), part.material),
            ItemKind::Pickaxe {
                handle,
                binding,
                head,
            } => {
                let handle = colorize_template(
                    images.get(&handles.pickaxe_handle_layer).unwrap().clone(),
                    material_color(handle.material),
                );

                let binding = colorize_template(
                    images.get(&handles.pickaxe_binding_layer).unwrap().clone(),
                    material_color(binding.material),
                );

                let head = colorize_template(
                    images.get(&handles.pickaxe_head_layer).unwrap().clone(),
                    material_color(head.material),
                );

                let image = copy_non_transparent_pixels(
                    copy_non_transparent_pixels(handle, &binding, 0, 0),
                    &head,
                    0,
                    0,
                );

                let handle = images.add(image);
                self.images.insert(item, handle.clone());
                return handle;
            }
            ItemKind::Shovel {
                handle,
                binding,
                head,
            } => {
                let handle = colorize_template(
                    images.get(&handles.shovel_handle_layer).unwrap().clone(),
                    material_color(handle.material),
                );

                let binding = colorize_template(
                    images.get(&handles.shovel_binding_layer).unwrap().clone(),
                    material_color(binding.material),
                );

                let head = colorize_template(
                    images.get(&handles.shovel_head_layer).unwrap().clone(),
                    material_color(head.material),
                );

                let image = copy_non_transparent_pixels(
                    copy_non_transparent_pixels(handle, &binding, 0, 0),
                    &head,
                    0,
                    0,
                );

                let handle = images.add(image);
                self.images.insert(item, handle.clone());
                return handle;
            }
            ItemKind::Hatchet {
                handle,
                binding,
                head,
            } => {
                let handle = colorize_template(
                    images.get(&handles.hatchet_handle_layer).unwrap().clone(),
                    material_color(handle.material),
                );

                let binding = colorize_template(
                    images.get(&handles.hatchet_binding_layer).unwrap().clone(),
                    material_color(binding.material),
                );

                let head = colorize_template(
                    images.get(&handles.hatchet_head_layer).unwrap().clone(),
                    material_color(head.material),
                );

                let image = copy_non_transparent_pixels(
                    copy_non_transparent_pixels(handle, &binding, 0, 0),
                    &head,
                    0,
                    0,
                );

                let handle = images.add(image);
                self.images.insert(item, handle.clone());
                return handle;
            }
            ItemKind::SmallBottle => {
                let template = images.get(&handles.small_bottle).unwrap().clone();
                let handle = images.add(colorize_template(template, Color::NONE));
                self.images.insert(item, handle.clone());
                return handle;
            }
            ItemKind::MediumBottle => {
                let template = images.get(&handles.medium_bottle).unwrap().clone();
                let handle = images.add(colorize_template(template, Color::NONE));
                self.images.insert(item, handle.clone());
                return handle;
            }
            ItemKind::LargeBottle => {
                let template = images.get(&handles.large_bottle).unwrap().clone();
                let handle = images.add(colorize_template(template, Color::NONE));
                self.images.insert(item, handle.clone());
                return handle;
            }
        };

        let template = images.get(&handle).unwrap().clone();
        let handle = images.add(colorize_template(template, material_color(material)));
        self.images.insert(item, handle.clone());
        handle
    }
}

fn material_color(material: Material) -> Color {
    match material {
        Material::Twig => Color::srgb(0.7, 0.45, 0.0),
        Material::PlantFiber => Color::srgb(0.1, 0.8, 0.1),
        Material::Flint => Color::srgb(0.4, 0.4, 0.4),
        Material::Glass => Color::srgb(0.83, 0.99, 1.0),
    }
}

fn colorize_template(mut template: Image, color: Color) -> Image {
    let color = color.to_srgba();

    for x in 0..template.width() {
        for y in 0..template.height() {
            let grayscale = template.get_color_at(x, y).unwrap().to_srgba();

            if grayscale.red != grayscale.green
                || grayscale.red != grayscale.blue
                || grayscale.green != grayscale.blue
            {
                continue;
            }

            let new_pixel = colorize_pixel(
                [grayscale.red, grayscale.green, grayscale.blue],
                [color.red, color.green, color.blue],
            );

            template
                .set_color_at(
                    x,
                    y,
                    Color::srgba(
                        new_pixel[0],
                        new_pixel[1],
                        new_pixel[2],
                        grayscale.alpha * color.alpha,
                    ),
                )
                .unwrap();
        }
    }

    template
}

fn colorize_pixel(grayscale: [f32; 3], color: [f32; 3]) -> [f32; 3] {
    let luminance = 0.299 * grayscale[0] + 0.587 * grayscale[1] + 0.114 * grayscale[2];
    [
        color[0] * luminance,
        color[1] * luminance,
        color[2] * luminance,
    ]
}

fn copy_non_transparent_pixels(
    mut image: Image,
    from: &Image,
    offset_x: u32,
    offset_y: u32,
) -> Image {
    for x in 0..from.width() {
        for y in 0..from.height() {
            let over = from.get_color_at(x, y).unwrap().to_srgba();

            let dest_x = x + offset_x;
            let dest_y = y + offset_y;

            // Skip if destination coordinates are out of bounds
            if dest_x >= image.width() || dest_y >= image.height() {
                continue;
            }

            let under = image.get_color_at(dest_x, dest_y).unwrap().to_srgba();

            // Standard "over" alpha compositing
            let a_out = over.alpha + under.alpha * (1.0 - over.alpha);

            let blended = if a_out > 0.0 {
                Color::srgba(
                    (over.red * over.alpha + under.red * under.alpha * (1.0 - over.alpha)) / a_out,
                    (over.green * over.alpha + under.green * under.alpha * (1.0 - over.alpha))
                        / a_out,
                    (over.blue * over.alpha + under.blue * under.alpha * (1.0 - over.alpha))
                        / a_out,
                    a_out,
                )
            } else {
                Color::NONE
            };

            image.set_color_at(dest_x, dest_y, blended).unwrap();
        }
    }

    image
}
