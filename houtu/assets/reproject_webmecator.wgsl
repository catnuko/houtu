@group(0) @binding(0) var texture_input: texture_2d<f32>;
@group(0) @binding(1) var mySampler : sampler;
@group(0) @binding(2) var<uniform> params: ParamsUniforms;

struct ParamsUniforms {
    viewportOrthographic: mat4x4<f32>,
    textureDimensions: vec2<f32>,
};
struct VertexInput {
  @location(0) position: vec4<f32>,
  @location(1) webMercatorT: vec2<f32>,
}
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2(in.position.x, in.webMercatorT.x);
    out.position = params.viewportOrthographic * (in.position * vec4<f32>(params.textureDimensions.x, params.textureDimensions.y, 1.0, 1.0));
    return out;
}
@fragment
fn fragment_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(texture_input, mySampler, uv);
}