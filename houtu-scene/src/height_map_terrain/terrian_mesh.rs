use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::Vec3,
    render::{mesh::Indices, primitives::Aabb},
};

use crate::{
    decompress_texture_coordinates, BoundingSphere, Matrix4, OrientedBoundingBox, TerrainEncoding,
    TerrainQuantization,
};
use bevy::prelude::*;
#[derive(Default, Clone, Debug, Component)]
pub struct TerrainMesh {
    pub center: DVec3,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub index_count_without_skirts: Option<u32>,
    pub vertex_count_without_skirts: u32,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub bounding_sphere_3d: BoundingSphere,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub vertex_stride: u32,
    pub oriented_bounding_box: OrientedBoundingBox,
    pub encoding: TerrainEncoding,
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
}
impl TerrainMesh {
    pub fn new(
        center: DVec3,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        index_count_without_skirts: Option<u32>,
        vertex_count_without_skirts: u32,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        bounding_sphere_3d: BoundingSphere,
        occludee_point_in_scaled_space: Option<DVec3>,
        vertex_stride: u32,
        oriented_bounding_box: OrientedBoundingBox,
        encoding: TerrainEncoding,
        west_indices_south_to_north: Vec<u32>,
        south_indices_east_to_west: Vec<u32>,
        east_indices_north_to_south: Vec<u32>,
        north_indices_west_to_east: Vec<u32>,
    ) -> Self {
        Self {
            center,
            vertices,
            indices,
            index_count_without_skirts,
            vertex_count_without_skirts,
            minimum_height,
            maximum_height,
            bounding_sphere_3d,
            occludee_point_in_scaled_space,
            vertex_stride,
            oriented_bounding_box,
            encoding,
            west_indices_south_to_north,
            south_indices_east_to_west,
            east_indices_north_to_south,
            north_indices_west_to_east,
        }
    }
}
impl From<&TerrainMesh> for Mesh {
    fn from(terrain_mesh: &TerrainMesh) -> Self {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(terrain_mesh.indices.clone())));
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        // let mut normals: Vec<[f32; 3]> = Vec::new();

        terrain_mesh
            .vertices
            .iter()
            .enumerate()
            .step_by(terrain_mesh.encoding.stride as usize)
            .for_each(|(i, x)| {
                if terrain_mesh.encoding.quantization == TerrainQuantization::NONE {
                    positions.push([
                        terrain_mesh.vertices[i] as f32,     //x
                        terrain_mesh.vertices[i + 1] as f32, //y
                        terrain_mesh.vertices[i + 2] as f32, //z
                                                             // terrain_mesh.vertices[i + 3] as f32, //height
                    ]);
                    uvs.push([
                        terrain_mesh.vertices[i + 4] as f32, //u
                        terrain_mesh.vertices[i + 5] as f32, //v
                    ]);
                    // normals.push([
                    //     terrain_mesh.vertices[i + 7] as f32, //u
                    //     terrain_mesh.vertices[i + 8] as f32, //v
                    //     terrain_mesh.vertices[i + 9] as f32, //v
                    // ])
                } else {
                    let xy = decompress_texture_coordinates(terrain_mesh.vertices[i] as f64);
                    let zh = decompress_texture_coordinates(terrain_mesh.vertices[i + 1] as f64);
                    let uv = decompress_texture_coordinates(terrain_mesh.vertices[i + 2] as f64);
                    let mut p = DVec3::new(xy.x, xy.y, zh.x);
                    p = terrain_mesh.encoding.fromScaledENU.multiply_by_point(&p);
                    positions.push([p.x as f32, p.y as f32, p.z as f32]);
                    uvs.push([uv.x as f32, uv.y as f32]);
                }
            });
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}
