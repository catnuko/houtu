use bevy::{prelude::*, render::mesh::Indices};
use houtu_scene::TerrainMesh;

pub struct TerrainMeshWarp(pub TerrainMesh);
impl From<TerrainMeshWarp> for Mesh {
    fn from(terrain_mesh_wrap: TerrainMeshWarp) -> Self {
        let terrain_mesh = terrain_mesh_wrap.0;

        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(terrain_mesh.indices)));
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
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
                                                         // terrain_mesh.vertices[i + 3] as f32, //height
                ]);
                uvs.push([
                    terrain_mesh.vertices[i + 4] as f32, //u
                    terrain_mesh.vertices[i + 5] as f32, //v
                ])
            });
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        mesh
    }
}
