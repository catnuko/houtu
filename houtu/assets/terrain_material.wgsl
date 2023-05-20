#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions
//TODO: 如果计算分割瓦片网格时减去瓦片的中心center_3d，再在shader中加上center_3d，则部分瓦片不显示，暂时不清楚原因，所以网片网格不减去center_3d了。
//如果有人知道，请告知。
struct TerrainMaterial {
    color: vec4<f32>,
    center_3d: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> material: TerrainMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position =  mesh_position_world_to_clip(vec4<f32>(vertex.position,1.0));
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}
