use bevy::prelude::*;
use wgpu::PrimitiveTopology;

use crate::oriented_bounding_box::OrientedBoundingBox;

pub struct Box3d {
    minimum: Vec3,
    maximum: Vec3,
}
impl From<Box3d> for Mesh {
    fn from(value: Box3d) -> Self {
        //实现功能
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut add_face = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| {
            let n = (b - a).cross(c - a).normalize();
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
            Vec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Vec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
        );
        add_face(
            Vec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
            Vec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Vec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            Vec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
            Vec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            Vec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Vec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
        );
        add_face(
            Vec3::new(value.minimum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.minimum.x, value.maximum.y, value.minimum.z),
            Vec3::new(value.minimum.x, value.maximum.y, value.maximum.z),
            Vec3::new(value.minimum.x, value.minimum.y, value.maximum.z),
        );
        add_face(
            Vec3::new(value.maximum.x, value.minimum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.minimum.z),
            Vec3::new(value.maximum.x, value.maximum.y, value.maximum.z),
            Vec3::new(value.maximum.x, value.minimum.y, value.maximum.z),
        );
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        mesh.insert_attribute(
            bevy::render::mesh::Mesh::ATTRIBUTE_POSITION,
            vertices
                .iter()
                .map(|v| [v.x, v.y, v.z])
                .collect::<Vec<[f32; 3]>>(),
        );
        mesh
    }
}
impl Box3d {
    // pub fn fromAxisAlignedBoundingBox(value: bevy::render::primitives::Aabb) -> Self {}
    pub fn fromDimensions(dimensions: Vec3) -> Self {
        let corner = dimensions * 0.5;
        Box3d {
            minimum: -corner,
            maximum: corner,
        }
    }
    pub fn from_center_halfaxes(center: Vec3, halfaxes: Mat3) -> Self {
        Box3d {
            minimum: center - halfaxes * Vec3::ONE,
            maximum: center + halfaxes * Vec3::ONE,
        }
    }
    pub fn frmo_obb(obb: OrientedBoundingBox) -> Self {
        Box3d::from_center_halfaxes(obb.center, obb.halfAxes)
    }
}
