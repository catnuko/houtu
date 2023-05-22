#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions
//TODO: 如果计算分割瓦片网格时减去瓦片的中心center_3d，再在shader中加上center_3d，则部分瓦片不显示，暂时不清楚原因，所以网片网格不减去center_3d了。
//如果有人知道，请告知。
struct TerrainMaterial {
    color: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> material: TerrainMaterial;
@group(1) @binding(1)
var image_texture: texture_2d<f32>;
@group(1) @binding(2)
var image_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>,
    // @location(1) normal: vec3<f32>,
    @location(2) uv:vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv:vec2<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position =  mesh_position_world_to_clip(vec4<f32>(in.position,1.0));
    out.uv = vec2<f32>(in.uv[0],1.0-in.uv[1]);
    return out;
}
struct FragmentInput{
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}
@fragment
fn fragment(in:FragmentInput) -> @location(0) vec4<f32> {
    // return material.color;
    var img = textureSample(image_texture, image_sampler,in.uv);
    if img.x!=1.&&img.y!=1.&&img.y!=1.{
        return img;
    }else{
        return material.color;
    }
    // return material.color * ;
}
