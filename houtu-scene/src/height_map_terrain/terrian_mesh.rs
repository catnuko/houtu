use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::Vec3,
    render::primitives::Aabb,
};

use crate::{BoundingSphere, OrientedBoundingBox, TerrainEncoding};
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
