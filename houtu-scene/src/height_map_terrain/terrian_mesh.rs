use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::Vec3,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        primitives::Aabb,
        render_resource::VertexFormat,
    },
};

use crate::{
    decompress_texture_coordinates, BoundingSphere, Matrix4, OrientedBoundingBox, TerrainEncoding,
    TerrainQuantization,
};
use bevy::prelude::*;
#[derive(Default, Clone, Debug, Component)]
pub struct TerrainMesh {
    pub center: DVec3,
    pub positions: Vec<DVec3>,
    pub indices: Vec<u32>,
    pub index_count_without_skirts: Option<u32>,
    pub vertex_count_without_skirts: u32,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub bounding_sphere_3d: BoundingSphere,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    // pub vertex_stride: u32,
    pub oriented_bounding_box: OrientedBoundingBox,
    // pub encoding: TerrainEncoding,
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
    pub heights: Vec<f64>,
    pub uvs: Vec<DVec2>,
    pub web_mecator_t: Vec<f64>,
    pub geodetic_surface_normals: Vec<DVec3>,
}
impl TerrainMesh {
    pub fn new(
        center: DVec3,
        positions: Vec<DVec3>,
        indices: Vec<u32>,
        index_count_without_skirts: Option<u32>,
        vertex_count_without_skirts: u32,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        bounding_sphere_3d: BoundingSphere,
        occludee_point_in_scaled_space: Option<DVec3>,
        // vertex_stride: u32,
        oriented_bounding_box: OrientedBoundingBox,
        // encoding: TerrainEncoding,
        west_indices_south_to_north: Vec<u32>,
        south_indices_east_to_west: Vec<u32>,
        east_indices_north_to_south: Vec<u32>,
        north_indices_west_to_east: Vec<u32>,
        heights: Vec<f64>,
        uvs: Vec<DVec2>,
        web_mecator_t: Vec<f64>,
        geodetic_surface_normals: Vec<DVec3>,
    ) -> Self {
        Self {
            center,
            positions,
            indices,
            index_count_without_skirts,
            vertex_count_without_skirts,
            minimum_height,
            maximum_height,
            bounding_sphere_3d,
            occludee_point_in_scaled_space,
            // vertex_stride,
            oriented_bounding_box,
            // encoding,
            west_indices_south_to_north,
            south_indices_east_to_west,
            east_indices_north_to_south,
            north_indices_west_to_east,
            heights,
            uvs,
            web_mecator_t,
            geodetic_surface_normals,
        }
    }
}
impl From<&TerrainMesh> for Mesh {
    fn from(terrain_mesh: &TerrainMesh) -> Self {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(terrain_mesh.indices.clone())));
        let positions: Vec<[f32; 3]> = terrain_mesh
            .positions
            .iter()
            .map(|v| v.as_vec3().into())
            .collect();
        let uvs: Vec<[f32; 2]> = terrain_mesh
            .uvs
            .iter()
            .map(|v| v.as_vec2().into())
            .collect();

        let web_mercator_t: Vec<f32> = terrain_mesh.web_mecator_t.iter().map(|x| *x as f32).collect();
        // bevy::log::info!(
        //     "{},{},{}",
        //     positions.len(),
        //     uvs.len(),
        //     web_mercator_t_list.len()
        // );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(
            MeshVertexAttribute::new("Vertex_WebMercatorT", 15, VertexFormat::Float32),
            web_mercator_t,
        );

        mesh
    }
}
