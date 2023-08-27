use bevy::{
    math::{DVec3},
};

use crate::{
    BoundingSphere, OrientedBoundingBox, TerrainEncoding,
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
