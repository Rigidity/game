#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;
@group(2) @binding(0) var array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var array_texture_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) packed: u32,
}

struct VertexOutput {
   @builtin(position) clip_position: vec4<f32>,
   @location(0) world_position: vec4<f32>,
   @location(1) uv: vec2<f32>,
   @location(2) tex_index: u32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let x = f32((vertex.packed >> 28) & 0xF);
    let y = f32((vertex.packed >> 24) & 0xF);
    let z = f32((vertex.packed >> 20) & 0xF);
    let corner = (vertex.packed >> 18) & 0x3;
    let face = (vertex.packed >> 15) & 0x7;
    let tex_index = vertex.packed & 0x7FFF;

    var final_pos = vec3<f32>(x, y, z);
    
    // Apply offsets based on face and corner
    switch face {
        case 0u: { // Top
            final_pos.x += f32(corner & 1u);
            final_pos.z += f32(corner >> 1u);
            final_pos.y += 1.0;
        }
        case 1u: { // Bottom
            final_pos.x += f32(corner & 1u);
            final_pos.z += f32(corner >> 1u);
        }
        case 2u: { // Left
            final_pos.z += f32(corner & 1u);
            final_pos.y += f32(corner >> 1u);
        }
        case 3u: { // Right
            final_pos.z += f32(corner & 1u);
            final_pos.y += f32(corner >> 1u);
            final_pos.x += 1.0;
        }
        case 4u: { // Front
            final_pos.x += f32(corner & 1u);
            final_pos.y += f32(corner >> 1u);
            final_pos.z += 1.0;
        }
        case 5u: { // Back
            final_pos.x += f32(corner & 1u);
            final_pos.y += f32(corner >> 1u);
        }
        default: {}
    }

    let uv = vec2<f32>(
        f32(corner & 1u),
        f32(corner >> 1u)
    );

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(final_pos, 1.0),
    );
    out.world_position = vec4<f32>(final_pos, 1.0);
    out.uv = uv;
    out.tex_index = tex_index;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let mip_level = calculate_mip_level(in.world_position);
    return sampleDownscaled(array_texture, array_texture_sampler, in.uv, i32(in.tex_index), mip_level);
}

fn calculate_mip_level(world_pos: vec4<f32>) -> u32 {
    // Calculate distance from camera using Bevy's built-in View uniform
    let distance = length(view.world_position.xyz - world_pos.xyz);
    
    // Using log2 for smooth transitions based on distance
    let level = floor(log2(max(distance / 20.0, 1.0)));
    return min(4u, u32(level)); // Clamp to max level (16x16)
}

fn sampleDownscaled(texture: texture_2d_array<f32>, my_sampler: sampler, uv: vec2<f32>, tex_index: i32, mip_level: u32) -> vec4<f32> {
    // Get the texture dimensions
    let dims = textureDimensions(texture);
    
    // Calculate block size based on mip level
    // mip_level 0 = 1x1 (original)
    // mip_level 1 = 2x2
    // mip_level 2 = 4x4
    // mip_level 3 = 8x8
    // mip_level 4 = 16x16
    let block_size = u32(1) << mip_level;
    
    // Calculate the size of one pixel in UV space
    let pixel_size = 1.0 / vec2<f32>(dims);
    
    // Calculate the center of the block we want to sample
    // Snap to grid by rounding down to multiples of block_size
    let block_uv = floor(uv * vec2<f32>(dims) / f32(block_size)) * f32(block_size) / vec2<f32>(dims);
    
    // Initialize accumulator for average
    var color_sum = vec4<f32>(0.0);
    let total_samples = block_size * block_size;
    
    // Sample all pixels in the block
    for (var y = 0u; y < block_size; y = y + 1u) {
        for (var x = 0u; x < block_size; x = x + 1u) {
            let sample_uv = block_uv + vec2<f32>(f32(x), f32(y)) * pixel_size;
            color_sum = color_sum + textureSample(texture, my_sampler, sample_uv, tex_index);
        }
    }
    
    // Return the average color
    return color_sum / f32(total_samples);
}
