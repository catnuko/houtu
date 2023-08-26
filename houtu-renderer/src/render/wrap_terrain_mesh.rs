use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::{Mesh, Vec3},
    render::{
        mesh::{Indices, MeshVertexAttribute},
        primitives::Aabb,
        render_resource::VertexFormat,
    },
};
use houtu_scene::{decompress_texture_coordinates, Matrix4, TerrainMesh, TerrainQuantization};

use super::terrian_material::TerrainMeshMaterial;

pub struct WrapTerrainMesh<'a>(pub &'a TerrainMesh);
impl<'a> WrapTerrainMesh<'a> {
}
impl<'a> From<WrapTerrainMesh<'a>> for Mesh {
    fn from(wrap_terrain_mesh: WrapTerrainMesh) -> Self {
        let terrain_mesh = wrap_terrain_mesh.0;
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(terrain_mesh.indices.clone())));
        let mut positions: Vec<[f32; 4]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut web_mercator_t: Vec<f32> = Vec::new();
        if terrain_mesh.encoding.quantization == TerrainQuantization::NONE {
            terrain_mesh
                .vertices
                .iter()
                .enumerate()
                .step_by(terrain_mesh.encoding.stride as usize)
                .for_each(|(i, x)| {
                    positions.push([
                        terrain_mesh.vertices[i] as f32,     //x
                        terrain_mesh.vertices[i + 1] as f32, //y
                        terrain_mesh.vertices[i + 2] as f32, //z
                        terrain_mesh.vertices[i + 3] as f32, //height
                    ]);
                    uvs.push([
                        terrain_mesh.vertices[i + 4] as f32, //u
                        terrain_mesh.vertices[i + 5] as f32, //v
                    ]);
                    if terrain_mesh.encoding.has_web_mercator_t {
                        web_mercator_t.push(terrain_mesh.vertices[i + 6] as f32);
                    }
                });
            mesh.insert_attribute(TerrainMeshMaterial::ATTRIBUTE_POSITION_HEIGHT, positions);
            // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            if terrain_mesh.encoding.has_web_mercator_t {
                mesh.insert_attribute(
                    TerrainMeshMaterial::ATTRIBUTE_WEB_MERCATOR_T,
                    web_mercator_t,
                );
            }
            mesh
        } else {
            let mut compressed0: Vec<[f32; 4]> = Vec::new();
            let mut compressed1: Vec<f32> = Vec::new();
            terrain_mesh
                .vertices
                .iter()
                .enumerate()
                .step_by(terrain_mesh.encoding.stride as usize)
                .for_each(|(i, x)| {
                    compressed0.push([
                        terrain_mesh.vertices[i] as f32,
                        terrain_mesh.vertices[i + 1] as f32,
                        terrain_mesh.vertices[i + 2] as f32,
                        if terrain_mesh.encoding.has_web_mercator_t {
                            terrain_mesh.vertices[i + 3] as f32
                        } else {
                            0f32
                        },
                    ]);
                    if terrain_mesh.encoding.has_vertex_normals {
                        compressed1.push(terrain_mesh.vertices[i + 4] as f32);
                    }
                });
            mesh.insert_attribute(TerrainMeshMaterial::COMPRESSED_0, compressed0);
            // mesh.insert_attribute(TerrainMeshMaterial::COMPRESSED_1, compressed1);
            mesh
        }
    }
}
