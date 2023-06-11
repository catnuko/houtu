#version 450

//
// Inputs
//
layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(location = 3) in float in_patch_scale; // Instanced
layout(location = 4) in vec3 in_patch_origin; // Instanced
layout(location = 5) in ivec4 in_neighbour_scales; // Instanced

//
// Uniforms
//
layout (std140, set = 0, binding = 0) uniform Projview {
    mat4 proj;
    mat4 view;
};

layout(std140, set = 1, binding = 0) uniform TerrainArgs {
    mat4 model;
    ivec2 terrain_size;
    float terrain_height_scale;
    float terrain_height_offset;
    bool wireframe;
};


// Outputs
// layout(location = 0) out VertexData {
//     vec4 position;
//     vec2 tex_coord;
// } vertex;
layout(location = 0) out VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    ivec4 neighbour_scales;
} vertex;
// layout(location = 0) out vec3 out_normal;
// layout(location = 1) out vec2 out_uv;
// layout(location = 2) out vec4 out_neighbour_scales;

vec2 calcTerrainTexCoord(in vec4 pos)
{
    return vec2(abs(pos.x - model[3][0]) / terrain_size.x, abs(pos.z - model[3][2]) / terrain_size.y);
}

void main()
{
    // Calcuate texture coordantes (u,v) relative to entire terrain
    vec4 vertex_position = model * vec4((in_pos * in_patch_scale) + in_patch_origin, 1.0);
    vertex.position = vertex_position.xyz;
    vertex.normal = mat3(model) * in_normal;
    vertex.neighbour_scales = in_neighbour_scales;
    vertex.tex_coord = calcTerrainTexCoord(vertex_position);
    gl_Position = vertex_position;

}