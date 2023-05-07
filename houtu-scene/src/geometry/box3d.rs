use bevy::math::DMat3;
use bevy::prelude::*;
use wgpu::PrimitiveTopology;

use super::OrientedBoundingBox;
use crate::coord::Cartesian3;

pub struct Box3d {
    minimum: Cartesian3,
    maximum: Cartesian3,
}
impl From<Box3d> for Mesh {
    fn from(value: Box3d) -> Self {
        //实现功能
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut add_face = |a: Cartesian3, b: Cartesian3, c: Cartesian3, d: Cartesian3| {
            let n = (b - a).cross(&(c - &a)).normalize();
            vertices.push(a);
            vertices.push(b);
            vertices.push(c);
            vertices.push(d);
            let i = vertices.len() as u32 - 4;
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + 2);
            indices.push(i);
            indices.push(i + 2);
            indices.push(i + 3);
        };
        add_face(
            Cartesian3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Cartesian3::new(value.minimum.x, value.maximum.y, value.minimum.z),
        );
        add_face(
            Cartesian3::new(value.minimum.x, value.minimum.y, value.maximum.z),
            Cartesian3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Cartesian3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            Cartesian3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            Cartesian3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            Cartesian3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Cartesian3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            Cartesian3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            Cartesian3::new(value.minimum.x, value.maximum.y, value.maximum.z),
            Cartesian3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            Cartesian3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Cartesian3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Cartesian3::new(value.maximum.x, value.minimum.y, value.maximum.z),
        );
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        // mesh.insert_attribute(
        //     bevy::render::mesh::Mesh::ATTRIBUTE_POSITION,
        //     vertices
        //         .iter()
        //         .map(|v| [v.x, v.y, v.z])
        //         .collect::<Vec<[f64; 3]>>(),
        // );
        mesh
    }
}
impl Box3d {
    // pub fn fromAxisAlignedBoundingBox(value: bevy::render::primitives::Aabb) -> Self {}
    pub fn fromDimensions(dimensions: Cartesian3) -> Self {
        let corner = dimensions * 0.5;
        Box3d {
            minimum: -corner,
            maximum: corner,
        }
    }
    pub fn from_center_halfaxes(center: Cartesian3, halfaxes: DMat3) -> Self {
        Box3d {
            minimum: center - halfaxes * Cartesian3::ONE.into(),
            maximum: center + halfaxes * Cartesian3::ONE.into(),
        }
    }
    pub fn frmo_obb(obb: OrientedBoundingBox) -> Self {
        Box3d::from_center_halfaxes(obb.center, obb.halfAxes)
    }
}
