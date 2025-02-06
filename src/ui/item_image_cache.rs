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
            _ => todo!(),
        };

        let color = match material {
            Material::Twig => Color::srgb(1.0, 0.7, 0.3),
            Material::PlantFiber => Color::srgb(0.1, 1.0, 0.1),
            Material::Flint => Color::srgb(0.5, 0.5, 0.5),
        };

        let template = images.get(&handle).unwrap().clone();
        let handle = images.add(colorize_template(template, color));
        self.images.insert(item, handle.clone());
        handle
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

// fn colorize_image(image: &mut DynamicImage, color: [u8; 3]) {
//     for Rgba(pixel) in image.as_mut_rgba8().unwrap().pixels_mut() {
//         let new_color = colorize_pixel([pixel[0], pixel[1], pixel[2]], color);
//         *pixel = [new_color[0], new_color[1], new_color[2], pixel[3]];
//     }
// }

/*
fn copy_non_transparent_pixels(
    image: &mut DynamicImage,
    from: &DynamicImage,
    offset_x: u32,
    offset_y: u32,
) {
    for (x, y, pixel) in from.pixels() {
        let dest_x = x + offset_x;
        let dest_y = y + offset_y;

        // Skip if destination coordinates are out of bounds
        if dest_x >= image.width() || dest_y >= image.height() {
            continue;
        }

        if pixel.0[3] == 0 || (pixel.0[3] < 255 && image.get_pixel(dest_x, dest_y).0[3] == 0) {
            continue;
        }

        // If pixel has any opacity
        let background = image.get_pixel(dest_x, dest_y);
        let alpha = pixel.0[3] as f32 / 255.0;

        // Blend each color channel (RGB)
        let blended = Rgba([
            blend_channel(pixel.0[0], background.0[0], alpha),
            blend_channel(pixel.0[1], background.0[1], alpha),
            blend_channel(pixel.0[2], background.0[2], alpha),
            blend_opacity(pixel.0[3], background.0[3]),
        ]);

        image.put_pixel(dest_x, dest_y, blended);
    }
}

// Helper function to blend a single color channel
fn blend_channel(foreground: u8, background: u8, alpha: f32) -> u8 {
    let fg = foreground as f32;
    let bg = background as f32;
    (fg * alpha + bg * (1.0 - alpha)) as u8
}

// Helper function to blend opacity values
fn blend_opacity(foreground: u8, background: u8) -> u8 {
    let alpha_f = foreground as f32 / 255.0;
    let alpha_b = background as f32 / 255.0;
    ((alpha_f + alpha_b * (1.0 - alpha_f)) * 255.0) as u8
}
 */
