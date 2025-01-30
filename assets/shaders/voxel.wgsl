#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

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
    @location(3) ao: f32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let x = f32((vertex.packed >> 28) & 0xF);
    let y = f32((vertex.packed >> 24) & 0xF);
    let z = f32((vertex.packed >> 20) & 0xF);
    let corner = (vertex.packed >> 18) & 0x3;
    let face = (vertex.packed >> 15) & 0x7;
    let ao = f32((vertex.packed >> 13) & 0x3) / 3.0;  // Unpack AO from bits 13-14
    let tex_index = vertex.packed & 0x1FFF;  // Get remaining 13 bits for tex_index

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
        1.0 - f32(corner >> 1u)
    );

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(final_pos, 1.0),
    );
    out.world_position = vec4<f32>(final_pos, 1.0);
    out.uv = uv;
    out.tex_index = tex_index;
    out.ao = ao;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ao_factor = mix(0.3, 1.0, in.ao);
    let texture_sample = textureSample(array_texture, array_texture_sampler, in.uv, i32(in.tex_index));
    
    // Discard fully transparent pixels
    if (texture_sample.a <= 0.1) {
        discard;
    }
    
    // For semi-transparent pixels (like leaves), use a higher alpha threshold
    let alpha = select(texture_sample.a, 0.8, texture_sample.a < 0.9);
    return vec4<f32>(texture_sample.rgb * ao_factor, alpha);
}
