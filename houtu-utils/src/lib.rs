use bevy::prelude::*;
pub fn getPointsFromMesh(mesh: &Mesh) -> Vec<Vec3> {
    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|p| Vec3::from(*p))
        .collect::<Vec<Vec3>>()
}
pub fn arrayToFloat32x3(points: &Vec<f32>) -> Vec<[f32; 3]> {
    let mut endPositions: Vec<[f32; 3]> = Vec::new();
    points
        .iter()
        .enumerate()
        .step_by(3)
        .for_each(|(i, x)| endPositions.push([points[i], points[i + 1], points[i + 2]]));
    return endPositions;
}
