use bevy::math::{DMat3, DVec3};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use super::OrientedBoundingBox;

pub struct Box3d {
    minimum: DVec3,
    maximum: DVec3,
}
impl From<Box3d> for Mesh {
    fn from(value: Box3d) -> Self {
        //实现功能
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut add_face = |a: DVec3, b: DVec3, c: DVec3, d: DVec3| {
            let _n = (b - a).cross(c - a).normalize();
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
            DVec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            DVec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
        );
        add_face(
            DVec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
            DVec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            DVec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            DVec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            DVec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            DVec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            DVec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            DVec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            DVec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
            DVec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            DVec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            DVec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            DVec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
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
    pub fn fromDimensions(dimensions: DVec3) -> Self {
        let corner = dimensions * 0.5;
        Box3d {
            minimum: -corner,
            maximum: corner,
        }
    }
    pub fn from_center_halfaxes(center: DVec3, halfaxes: DMat3) -> Self {
        Box3d {
            minimum: center - halfaxes * DVec3::ONE,
            maximum: center + halfaxes * DVec3::ONE,
        }
    }
    pub fn frmo_obb(obb: OrientedBoundingBox) -> Self {
        Box3d::from_center_halfaxes(obb.center, obb.half_axes)
    }
}
