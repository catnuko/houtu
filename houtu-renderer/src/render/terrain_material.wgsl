// #import bevy_pbr::mesh_view_bindings
// #import bevy_pbr::mesh_bindings
// #import bevy_pbr::mesh_functions

#ifdef QUANTIZATION_BITS12
struct Vertex {
    @location(0) compressed0: vec4<f32>,
    // @location(1) compressed1: f32,
};

#else
struct Vertex {
    @location(0) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) web_mercator_t: f32,
};
#endif
struct VertexUniform {
    minimum_height: f32,
    maximum_height: f32,
    center_3d: vec3<f32>,
    scale_and_bias_x: vec4<f32>,
    scale_and_bias_y: vec4<f32>,
    scale_and_bias_z: vec4<f32>,
    scale_and_bias_w: vec4<f32>,
    mvp_x: vec4<f32>,
    mvp_y: vec4<f32>,
    mvp_z: vec4<f32>,
    mvp_w: vec4<f32>,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) height: f32,
    @location(1) texture_coordinates: vec3<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    #ifdef QUANTIZATION_BITS12
    let xy = czm_decompressTextureCoordinates(in.compressed0.x);
    let zh = czm_decompressTextureCoordinates(in.compressed0.y);
    var position = vec3<f32>(xy, zh.x);
    var height = zh.y;
    let scale_and_bias = mat4x4<f32>(vertex_uniform.scale_and_bias_x, vertex_uniform.scale_and_bias_y, vertex_uniform.scale_and_bias_z, vertex_uniform.scale_and_bias_w);
    let uv = czm_decompressTextureCoordinates(in.compressed0.z);
    height = height * (vertex_uniform.maximum_height - vertex_uniform.minimum_height) + vertex_uniform.minimum_height;
    position = (scale_and_bias * vec4(position, 1.)).xyz;
    position = position + vertex_uniform.center_3d;
    let web_mercator_t = czm_decompressTextureCoordinates(in.compressed0.w).x;
    let texture_coordinates = vec3<f32>(uv, web_mercator_t);
    #else
    var position = in.position.xyz;
    let height = in.position.w;
    let texture_coordinates = vec3<f32>(in.uv, in.web_mercator_t);
    position = position + vertex_uniform.center_3d;
    #endif
    // out.position = mesh_position_world_to_clip(vec4<f32>(position, 1.0));
    let mvp = mat4x4<f32>(vertex_uniform.mvp_x, vertex_uniform.mvp_y, vertex_uniform.mvp_z, vertex_uniform.mvp_w);
    let clip_position = mvp * vec4<f32>(position, 1.0);
    out.position = clip_position;
    out.texture_coordinates = texture_coordinates;
    out.height = height;
    return out;
}


struct FragmentInput {
    @location(0) height: f32,
    @location(1) texture_coordinates: vec3<f32>,
}

struct TerrainMaterialUniform {
    translation_and_scale: vec4<f32>,
    coordinate_rectangle: vec4<f32>,
    use_web_mercator_t: f32,
    alpha: f32,
    apply_day_night_alpha: f32,
    brightness: f32,
    contrast: f32,
    hue: f32,
    saturation: f32,
    one_over_gamma: f32,
};
struct StateUniform {
    texture_num: i32
 
}

@group(0) @binding(0) var texture_array: texture_2d_array<f32>;
@group(0) @binding(1) var my_sampler: sampler;
@group(0) @binding(2) var<storage,read> terrain_material_uniforms: array<TerrainMaterialUniform>;
@group(0) @binding(3) var<uniform> state_uniform: StateUniform;
@group(0) @binding(4) var<uniform> vertex_uniform: VertexUniform;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var final_color = vec4<f32>(0.0);
    var i: i32 = 0;
    loop{
        if i >= state_uniform.texture_num {
            break;
        }
        let u = terrain_material_uniforms[i];

        // let texture_to_sample = texture_array[i];
        let texture_sampler = my_sampler;
        let use_web_mercator_t = u.use_web_mercator_t;
        let clamped_texture_coordinates = clamp(in.texture_coordinates, vec3<f32>(0.0), vec3<f32>(1.0));
        let tile_texture_coordinates = select(clamped_texture_coordinates.xy, clamped_texture_coordinates.xz, use_web_mercator_t == 1.0);
        let texture_coordinate_rectangle = u.coordinate_rectangle;
        let translation_and_scale = u.translation_and_scale;
        var texture_alpha = 1.0;
        
        #ifdef APPLY_ALPHA
        texture_alpha = u.alpha;
        #endif

        var texture_night_alpha = 1.0;
        #ifdef APPLY_DAY_NIGHT_ALPHA
        texture_night_alpha = u.apply_day_night_alpha;
        #endif

        var texture_day_alpha = 1.0;
        #ifdef APPLY_DAY_NIGHT_ALPHA
        texture_day_alpha = u.apply_day_night_alpha;
        #endif

        var texture_brightness = 0.0;
        #ifdef APPLY_BRIGHTNESS
        texture_brightness = u.brightness;
        #endif

        var texture_contrast = 0.0;
        #ifdef APPLY_CONTRAST
        texture_contrast = u.contrast;
        #endif

        var texture_hue = 0.0;
        #ifdef APPLY_HUE
        texture_hue = u.hue;
        #endif

        var texture_saturation = 0.0;
        #ifdef APPLY_SATURATION
        texture_saturation = u.saturation;
        #endif

        var texture_one_over_gamma = 0.0;
        #ifdef APPLY_GAMMA
        texture_one_over_gamma = u.one_over_gamma;
        #endif

        // 不在texture_coordinate_rectangle内部的像素的alpha值都是0，这样能避免采样到图片的其它地方。
        // 因为texture_coordinate_rectangle的值是相对于地形瓦片而言，所以在对tile_texture_coordinates变换之前判断
        var alpha_multiplier = step(texture_coordinate_rectangle.xy, tile_texture_coordinates);
        texture_alpha = texture_alpha * alpha_multiplier.x * alpha_multiplier.y;

        alpha_multiplier = step(vec2<f32>(0.0), texture_coordinate_rectangle.zw - tile_texture_coordinates);
        texture_alpha = texture_alpha * alpha_multiplier.x * alpha_multiplier.y;

        let translation = translation_and_scale.xy;
        let scale = translation_and_scale.zw;
        let texture_coordinates = tile_texture_coordinates * scale + translation;
        let value = textureSample(texture_array, my_sampler, texture_coordinates.xy, i);
        var color = value.rgb;
        var alpha = value.a;


        #ifdef APPLY_BRIGHTNESS
        color = mix(vec3<f32>(0.0), color, texture_brightness);
        #endif

        #ifdef APPLY_CONTRAST
        color = mix(vec3<f32>(0.5), color, texture_contrast);
        #endif

        #ifdef APPLY_HUE
        color = czm_hue(color, texture_hue);
        #endif

        #ifdef APPLY_SATURATION
        color = czm_saturation(color, texture_saturation);
        #endif

        let source_alpha = alpha * texture_alpha;
        var out_alpha = mix(final_color.a, 1.0, source_alpha);
        out_alpha += sign(out_alpha) - 1.0;
        let out_color = mix(final_color.rgb * final_color.a, color, source_alpha) / out_alpha;
        final_color = vec4<f32>(out_color, max(out_alpha, 0.0));
        i++;
    }
    return final_color;
    // return vec4<f32>(1.0);
}

fn czm_hue(rgb: vec3<f32>, adjustment: f32) -> vec3<f32> {
    let toYIQ = mat3x3(0.299, 0.587, 0.114, 0.595716, -0.274453, -0.321263, 0.211456, -0.522591, 0.311135);
    let toRGB = mat3x3(1.0, 0.9563, 0.6210, 1.0, -0.2721, -0.6474, 1.0, -1.107, 1.7046);

    let yiq = toYIQ * rgb;
    let hue = atan2(yiq.z, yiq.y) + adjustment;
    let chroma = sqrt(yiq.z * yiq.z + yiq.y * yiq.y);

    let color = vec3(yiq.x, chroma * cos(hue), chroma * sin(hue));
    return toRGB * color;
}
fn czm_saturation(rgb: vec3<f32>, adjustment: f32) -> vec3<f32> {
    // Algorithm from Chapter 16 of OpenGL Shading Language
    let W = vec3<f32>(0.2125, 0.7154, 0.0721);
    let intensity = vec3<f32>(dot(rgb, W));
    return mix(intensity, rgb, adjustment);
}
fn czm_decompressTextureCoordinates(encoded: f32) -> vec2<f32> {
    let temp = encoded / 4096.0;
    let x_zero_to4095 = floor(temp);
    let stx = x_zero_to4095 / 4095.0;
    let sty = (encoded - x_zero_to4095 * 4096.0) / 4095.0;
    return vec2<f32>(stx, sty);
}
