#version 450

// Inputs
layout(quads, fractional_even_spacing, cw) in;

layout(location = 0) in vec3 in_normal[];
layout(location = 4) in vec2 in_uv[];
layout(location = 8) in float in_tesselation_level[];


// Uniforms
layout (std140, set = 0, binding = 0) uniform Projview {
    mat4 proj;
    mat4 view;
};
layout (std140, set = 1, binding = 0) uniform TerrainArgs {
    mat4 model;
    ivec2 terrain_size;
    float terrain_height_scale;
    float terrain_height_offset;
    bool wireframe;
};

// layout (std140, set = 0, binding = 1) uniform TessArgs {
//     vec2 viewport;
//     float terrain_height_scale;
//     float terrain_height_offset;
//     //   0
//     // 3 x 1
//     //   2
//     vec4 neighbour_scales;
// };
layout (set = 1, binding = 1) uniform sampler2D terrain_height_tex;


// Outputs
layout(location = 0) out vec3 out_normal;
layout(location = 1) out vec2 out_uv;
layout(location = 2) out vec3 out_view_vec;
layout(location = 3) out vec3 out_world_pos;
layout(location = 4) out float out_tesselation_level;



vec4 interpolate4(in vec4 v0, in vec4 v1, in vec4 v2, in vec4 v3)
{
    vec4 a = mix(v0, v1, gl_TessCoord.x);
    vec4 b = mix(v3, v2, gl_TessCoord.x);
    return mix(a, b, gl_TessCoord.y);
}
vec3 interpolate3(in vec3 v0, in vec3 v1, in vec3 v2, in vec3 v3)
{
	vec3 a = mix(v0, v1, gl_TessCoord.x);
	vec3 b = mix(v3, v2, gl_TessCoord.x);
	return mix(a, b, gl_TessCoord.y);
}

vec2 interpolate2(in vec2 v0, in vec2 v1, in vec2 v2, in vec2 v3)
{
	vec2 a = mix(v0, v1, gl_TessCoord.x);
	vec2 b = mix(v3, v2, gl_TessCoord.x);
	return mix(a, b, gl_TessCoord.y);
}


void main()
{
    // Calculate the vertex position using the four original points and interpolate depneding on the tessellation coordinates.	
    // tes.position = interpolate(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_in[2].gl_Position, gl_in[3].gl_Position);
    vec4 position = interpolate4(gl_in[0].gl_Position, gl_in[1].gl_Position, gl_in[2].gl_Position, gl_in[3].gl_Position);

    // Terrain heightmap coords
    out_uv = interpolate2(in_uv[0], in_uv[1], in_uv[2], in_uv[3]);

    out_normal = interpolate3(in_normal[0], in_normal[1], in_normal[2], in_normal[3]);

    // Sample the heightmap and offset y position of vertex
    vec4 samp = texture(terrain_height_tex, out_uv);
    // vec4 samp2 = texture(terrain_height_tex_2, terrain_tex_coord);
    position.y = samp.r * terrain_height_scale + terrain_height_offset;

    // Project the vertex to clip space and send it along
    // gl_Position = proj * view * position;
    gl_Position = position;

    out_view_vec = -position.xyz;

    out_world_pos = position.xyz;

    // tes.tex_coord = terrain_tex_coord;
    // tes.tesselation_level = tcs[0].tesselation_level;
    out_tesselation_level = in_tesselation_level[0];
}