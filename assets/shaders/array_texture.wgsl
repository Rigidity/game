#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

@group(2) @binding(0) var array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var array_texture_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) packed: u32,
}

struct VertexOutput {
   @builtin(position) clip_position: vec4<f32>,
   @location(0) uv: vec2<f32>,
   @location(1) tex_index: u32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
   let x = f32((vertex.packed >> 28) & 0xF);
   let y = f32((vertex.packed >> 24) & 0xF);
   let z = f32((vertex.packed >> 20) & 0xF);
   let corner = (vertex.packed >> 18) & 0x3;
   let tex_index = vertex.packed & 0x3FFFF;

   let uv = vec2<f32>(
       f32(corner & 1),
       f32(corner >> 1)
   );

   var out: VertexOutput;
   out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(x, y, z, 1.0),
    );
   out.uv = uv;
   out.tex_index = tex_index;
   return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
   return textureSample(array_texture, array_texture_sampler, in.uv, i32(in.tex_index));
}
