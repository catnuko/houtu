#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions
//TODO: 如果计算分割瓦片网格时减去瓦片的中心center_3d，再在shader中加上center_3d，则部分瓦片不显示，暂时不清楚原因，所以网片网格不减去center_3d了。

struct Vertex {
    @location(0) position: vec3<f32>,
    // @location(1) normal:vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) use_web_mercator_t: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec3<f32>,
};

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position = mesh_position_world_to_clip(vec4<f32>(in.position, 1.0));
    let uv = in.uv;
    out.texture_coordinates = vec3<f32>(uv.xy, in.use_web_mercator_t);
    return out;
}


struct FragmentInput {
    @location(0) texture_coordinates: vec3<f32>,
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

@group(1) @binding(0) var texture_array: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var my_sampler: sampler;
@group(1) @binding(2) var<storage,read> terrain_material_uniforms: array<TerrainMaterialUniform>;
@group(1) @binding(3) var<uniform> state_uniform: StateUniform;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var final_color = vec4<f32>(0.0);
    var i: i32 = 0;
    loop{
        if i >= state_uniform.texture_num {
            break;
        }
        let u = terrain_material_uniforms[i];

        let texture_to_sample = texture_array[i];
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
        // let texture_coordinates = tile_texture_coordinates;
        let value = textureSample(texture_array[i], my_sampler, texture_coordinates);
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
