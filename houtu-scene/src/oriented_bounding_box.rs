use std::f64::MAX;

use bevy::math::{DMat3, DVec3};
use bevy::prelude::*;

use crate::box3d::Box3d;
#[derive(Component)]
pub struct OrientedBoundingBox {
    pub center: DVec3,
    pub halfAxes: DMat3,
}
impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            halfAxes: DMat3::ZERO,
        }
    }
}
impl OrientedBoundingBox {
    pub fn fromPoints(positions: &[DVec3]) -> Self {
        let mut result = Self::default();
        let length = positions.len();
        if length == 0 {
            return result;
        }

        let mut meanPoint = positions[0].clone();
        for i in 1..length {
            meanPoint = meanPoint + positions[i];
        }
        let invLength = 1.0 / length as f64;
        meanPoint = meanPoint / invLength;

        let mut exx = 0.0;
        let mut exy = 0.0;
        let mut exz = 0.0;
        let mut eyy = 0.0;
        let mut eyz = 0.0;
        let mut ezz = 0.0;
        let mut p;
        for i in 0..length {
            p = positions[i] - meanPoint;
            exx += p.x * p.x;
            exy += p.x * p.y;
            exz += p.x * p.z;
            eyy += p.y * p.y;
            eyz += p.y * p.z;
            ezz += p.z * p.z;
        }

        exx *= invLength;
        exy *= invLength;
        exz *= invLength;
        eyy *= invLength;
        eyz *= invLength;
        ezz *= invLength;

        let covarianceMatrixSlice = [exx, exy, exz, exy, eyy, eyz, exz, eyz, ezz];
        let covarianceMatrix = DMat3::from_cols_array(&covarianceMatrixSlice);

        let eigenDecomposition = houtu_math::computeEigenDecomposition(covarianceMatrix);
        let rotation = eigenDecomposition.unitary.clone();
        result.halfAxes = rotation.clone();

        let mut v1 = rotation.col(0);
        let mut v2 = rotation.col(1);
        let mut v3 = rotation.col(2);

        let mut u1 = -MAX;
        let mut u2 = -MAX;
        let mut u3 = -MAX;
        let mut l1 = MAX;
        let mut l2 = MAX;
        let mut l3 = MAX;
        for i in 0..length {
            p = positions[i];
            u1 = v1.dot(p).max(u1);
            u2 = v2.dot(p).max(u2);
            u3 = v3.dot(p).max(u3);

            l1 = v1.dot(p).max(l1);
            l2 = v2.dot(p).max(l2);
            l3 = v3.dot(p).max(l3);
        }
        v1 = v1 * 0.5 * (l1 + u1);
        v2 = v2 * 0.5 * (l2 + u2);
        v3 = v3 * 0.5 * (l3 + u3);

        result.center = v1 + v2 + v3;
        let scale = DVec3::new(u1 - l1, u2 - l2, u3 - l3) * 0.5;
        result.halfAxes = houtu_math::vec3::multiplyByScalar(result.halfAxes, scale);

        result
    }
}
#[derive(Bundle)]
pub struct OrientedBoundingBoxBundle {
    pub obb: OrientedBoundingBox,
    pub visibility: Visibility,
}
pub struct OrientedBoundingBoxPlugin;
impl Default for OrientedBoundingBoxPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl bevy::app::Plugin for OrientedBoundingBoxPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // app.add_startup_system(setup);
    }
}
fn setup(
    mut commands: bevy::ecs::system::Commands,
    mut query: Query<&mut OrientedBoundingBox>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut obb) in query.iter_mut() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Box3d::from_center_halfaxes(obb.center, obb.halfAxes).into()),
            material: materials.add(Color::BLACK.into()),
            ..Default::default()
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const positions: [DVec3; 6] = [
        DVec3::new(2.0, 0.0, 0.0),
        DVec3::new(0.0, 3.0, 0.0),
        DVec3::new(0.0, 0.0, 4.0),
        DVec3::new(-2.0, 0.0, 0.0),
        DVec3::new(0.0, -3.0, 0.0),
        DVec3::new(0.0, 0.0, -4.0),
    ];

    #[test]
    fn init_work() {
        let obb = OrientedBoundingBox::default();
        assert_eq!(obb.center, DVec3::ZERO);
        assert_eq!(obb.halfAxes, DMat3::ZERO);
    }
    #[test]
    fn empty_points_work() {
        let points = vec![];
        let obb = OrientedBoundingBox::fromPoints(&points);
        assert_eq!(obb.center, DVec3::ZERO);
        assert_eq!(obb.halfAxes, DMat3::ZERO);
    }
    #[test]
    fn fromPointsCorrectScale() {
        let obb = OrientedBoundingBox::fromPoints(&positions);
        let scale = DVec3::new(2.0, 3.0, 4.0);
        let expect_mat3 = mat3_from_scale_vec3(obb.halfAxes, scale);
        assert_eq!(obb.halfAxes, expect_mat3);
        assert_eq!(obb.center, DVec3::ZERO);
    }
}
pub fn mat3_from_scale_vec3(matrix: DMat3, scale: DVec3) -> DMat3 {
    DMat3::from_cols(
        DVec3::new(scale.x, 0.0, 0.0),
        DVec3::new(0.0, scale.y, 0.0),
        DVec3::new(0.0, 0.0, scale.z),
    )
}
