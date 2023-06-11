#version 450


// Inputs
layout(location = 0) in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    ivec4 neighbour_scales;
} vertex[];

// layout(location = 0) in vec3 in_normal[];
// layout(location = 1) in vec2 in_uv[];
// layout(location = 2) in vec4 in_neighbour_scales[];

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
layout(vertices = 4) out;

layout(location = 0) out vec3 out_normal[4];
layout(location = 4) out vec2 out_uv[4];
layout(location = 8) out float out_tesselation_level[4];



void main()
{
    gl_TessLevelOuter[0] = max(2.0, vertex[gl_InvocationID].neighbour_scales.w);
    gl_TessLevelOuter[1] = max(2.0, vertex[gl_InvocationID].neighbour_scales.x);
    gl_TessLevelOuter[2] = max(2.0, vertex[gl_InvocationID].neighbour_scales.y);
    gl_TessLevelOuter[3] = max(2.0, vertex[gl_InvocationID].neighbour_scales.z);


    // Inner tessellation level
    gl_TessLevelInner[0] = 0.5 * (gl_TessLevelOuter[0] + gl_TessLevelOuter[3]);
    gl_TessLevelInner[1] = 0.5 * (gl_TessLevelOuter[2] + gl_TessLevelOuter[1]);

    // Pass the patch verts along
    gl_out[gl_InvocationID].gl_Position = gl_in[gl_InvocationID].gl_Position;

    out_normal[gl_InvocationID] = vertex[gl_InvocationID].normal;

    // Output heightmap coordinates
    out_uv[gl_InvocationID] = vertex[gl_InvocationID].tex_coord;

    // Output tessellation level (used for wireframe coloring)
    // tcs[gl_InvocationID].tesselation_level = gl_TessLevelOuter[0];
    out_tesselation_level[gl_InvocationID] = 0.5 * (gl_TessLevelInner[0] + gl_TessLevelInner[1]);
}