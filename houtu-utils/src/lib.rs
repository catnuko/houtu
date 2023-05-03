use bevy::math::{DMat3, DVec3};
use bevy::prelude::*;

pub fn getPointsFromMesh(mesh: &Mesh) -> Vec<DVec3> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|p| DVec3::from(*p))
        .collect::<Vec<DVec3>>()
}
pub fn arrayToFloat32x3(points: &Vec<f64>) -> Vec<[f64; 3]> {
    let mut endPositions: Vec<[f64; 3]> = Vec::new();
    points
        .iter()
        .enumerate()
        .step_by(3)
        .for_each(|(i, x)| endPositions.push([points[i], points[i + 1], points[i + 2]]));
    return endPositions;
}
