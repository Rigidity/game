#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    mesh_view_bindings::{globals, view}
}

struct BlockInteraction {
    x: u32,
    y: u32,
    z: u32,
    face: u32,
    value: u32,
}

@group(2) @binding(0) var array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var array_texture_sampler: sampler;
@group(2) @binding(2) var destroy_texture: texture_2d_array<f32>;
@group(2) @binding(3) var destroy_texture_sampler: sampler;
@group(2) @binding(4) var<uniform> block_interaction: BlockInteraction;

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
    @location(4) interaction: u32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let x_int = (vertex.packed >> 28) & 0xF;
    let y_int = (vertex.packed >> 24) & 0xF;
    let z_int = (vertex.packed >> 20) & 0xF;
    let x = f32(x_int);
    let y = f32(y_int);
    let z = f32(z_int);
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
    let model = get_world_from_local(vertex.instance_index);
    out.clip_position = mesh_position_local_to_clip(
        model,
        vec4<f32>(final_pos, 1.0),
    );
    out.world_position = model * vec4<f32>(final_pos, 1.0);
    out.uv = uv;
    out.tex_index = tex_index;
    out.ao = ao;

    out.interaction = select(
        0u,
        block_interaction.value,
        block_interaction.x == x_int
            && block_interaction.y == y_int
            && block_interaction.z == z_int
    );

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

    var final_color = texture_sample.rgb;

    // Apply destroy texture overlay if block is being broken
    if (in.interaction > 0u) {
         // Create a slower, more obvious pulsating effect
        let pulse = (sin(globals.time * 4.0) * 0.1) + 0.8;
        let interaction_factor = select(pulse, 1.0, in.interaction == 0u);
        final_color = final_color * ao_factor * interaction_factor;

        if (in.interaction > 1u) {
            let destroy_stage = in.interaction - 2u;  // Convert to 0-10 range
            let destroy_overlay = textureSample(destroy_texture, destroy_texture_sampler, in.uv, i32(destroy_stage));
            // Blend colors taking alpha into account
            final_color = final_color * (1.0 - destroy_overlay.a) + destroy_overlay.rgb * destroy_overlay.a;
        }
    }

    final_color *= ao_factor;

    // Calculate fog
    let fog_color = vec3<f32>(0.3, 0.6, 0.9);
    let fog_start = 80.0;
    let fog_end = 90.0;
    
    let distance = length(in.world_position.xyz - view.world_position.xyz);
    let fog_factor = clamp((distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    
    final_color = mix(final_color, fog_color, fog_factor);

    return vec4<f32>(final_color, alpha);
}
