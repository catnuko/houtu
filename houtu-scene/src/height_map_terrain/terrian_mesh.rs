use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::Vec3,
    render::{mesh::Indices, primitives::Aabb},
};

use crate::{BoundingSphere, OrientedBoundingBox, TerrainEncoding};
use bevy::prelude::*;
#[derive(Default, Clone, Debug, Component)]
pub struct TerrainMesh {
    pub center: DVec3,
    pub vertices: Vec<f64>,
    pub indices: Vec<u32>,
    pub indexCountWithoutSkirts: Option<u32>,
    pub vertexCountWithoutSkirts: u32,
    pub minimumHeight: f64,
    pub maximumHeight: f64,
    pub boundingSphere3D: BoundingSphere,
    pub occludeePointInScaledSpace: Option<DVec3>,
    pub vertexStride: u32,
    pub orientedBoundingBox: OrientedBoundingBox,
    pub encoding: TerrainEncoding,
    pub westIndicesSouthToNorth: Vec<u32>,
    pub southIndicesEastToWest: Vec<u32>,
    pub eastIndicesNorthToSouth: Vec<u32>,
    pub northIndicesWestToEast: Vec<u32>,
}
impl TerrainMesh {
    pub fn new(
        center: DVec3,
        vertices: Vec<f64>,
        indices: Vec<u32>,
        indexCountWithoutSkirts: Option<u32>,
        vertexCountWithoutSkirts: u32,
        minimumHeight: f64,
        maximumHeight: f64,
        boundingSphere3D: BoundingSphere,
        occludeePointInScaledSpace: Option<DVec3>,
        vertexStride: u32,
        orientedBoundingBox: OrientedBoundingBox,
        encoding: TerrainEncoding,
        westIndicesSouthToNorth: Vec<u32>,
        southIndicesEastToWest: Vec<u32>,
        eastIndicesNorthToSouth: Vec<u32>,
        northIndicesWestToEast: Vec<u32>,
    ) -> Self {
        Self {
            center,
            vertices,
            indices,
            indexCountWithoutSkirts,
            vertexCountWithoutSkirts,
            minimumHeight,
            maximumHeight,
            boundingSphere3D,
            occludeePointInScaledSpace,
            vertexStride,
            orientedBoundingBox,
            encoding,
            westIndicesSouthToNorth,
            southIndicesEastToWest,
            eastIndicesNorthToSouth,
            northIndicesWestToEast,
        }
    }
}
impl From<&TerrainMesh> for Mesh {
    fn from(terrain_mesh: &TerrainMesh) -> Self {
        let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(terrain_mesh.indices.clone())));
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
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
                ]);
                // normals.push([
                //     terrain_mesh.vertices[i + 7] as f32, //u
                //     terrain_mesh.vertices[i + 8] as f32, //v
                //     terrain_mesh.vertices[i + 9] as f32, //v
                // ])
            });
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}
