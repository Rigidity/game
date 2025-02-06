use bevy::{prelude::*, utils::HashMap};

use crate::{
    item::{Item, Material},
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

        let (handle, material) = match item {
            Item::Flint => return handles.flint.clone(),
            Item::Soil => return handles.soil.clone(),
            Item::Twig => return handles.twig.clone(),
            Item::PlantFiber => return handles.plant_fiber.clone(),
            Item::Handle(material) => (handles.handle.clone(), Material::from(material)),
            Item::Binding(material) => (handles.binding.clone(), Material::from(material)),
            Item::PickaxeHead(material) => (handles.pickaxe_head.clone(), Material::from(material)),
            Item::Pickaxe {
                handle,
                binding,
                head,
            } => {
                let handle = colorize_template(
                    images.get(&handles.pickaxe_handle_layer).unwrap().clone(),
                    material_color(handle.into()),
                );

                let binding = colorize_template(
                    images.get(&handles.pickaxe_binding_layer).unwrap().clone(),
                    material_color(binding.into()),
                );

                let head = colorize_template(
                    images.get(&handles.pickaxe_head_layer).unwrap().clone(),
                    material_color(head.into()),
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
        };

        let template = images.get(&handle).unwrap().clone();
        let handle = images.add(colorize_template(template, material_color(material)));
        self.images.insert(item, handle.clone());
        handle
    }
}

fn material_color(material: Material) -> Color {
    match material {
        Material::Twig => Color::srgb(1.0, 0.7, 0.3),
        Material::PlantFiber => Color::srgb(0.1, 1.0, 0.1),
        Material::Flint => Color::srgb(0.5, 0.5, 0.5),
    }
}

fn colorize_template(mut template: Image, color: Color) -> Image {
    let color = color.to_srgba();

    for x in 0..template.width() {
        for y in 0..template.height() {
            let grayscale = template.get_color_at(x, y).unwrap().to_srgba();
            let new_pixel = colorize_pixel(
                [grayscale.red, grayscale.green, grayscale.blue],
                [color.red, color.green, color.blue],
            );
            template
                .set_color_at(
                    x,
                    y,
                    Color::srgba(new_pixel[0], new_pixel[1], new_pixel[2], grayscale.alpha),
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
            let pixel = from.get_color_at(x, y).unwrap().to_srgba();

            let dest_x = x + offset_x;
            let dest_y = y + offset_y;

            // Skip if destination coordinates are out of bounds
            if dest_x >= image.width() || dest_y >= image.height() {
                continue;
            }

            let current = image.get_color_at(dest_x, dest_y).unwrap().to_srgba();

            if pixel.alpha == 0.0 || (pixel.alpha < 1.0 && current.alpha == 0.0) {
                continue;
            }

            // If pixel has any opacity

            // Blend each color channel (RGB)
            let blended = Color::srgba(
                blend_channel(pixel.red, current.red, pixel.alpha),
                blend_channel(pixel.green, current.green, pixel.alpha),
                blend_channel(pixel.blue, current.blue, pixel.alpha),
                blend_opacity(pixel.alpha, current.alpha),
            );

            image.set_color_at(dest_x, dest_y, blended).unwrap();
        }
    }

    image
}

// Helper function to blend a single color channel
fn blend_channel(foreground: f32, background: f32, alpha: f32) -> f32 {
    let fg = foreground;
    let bg = background;
    fg * alpha + bg * (1.0 - alpha)
}

// Helper function to blend opacity values
fn blend_opacity(foreground: f32, background: f32) -> f32 {
    let alpha_f = foreground;
    let alpha_b = background;
    alpha_f + alpha_b * (1.0 - alpha_f)
}
