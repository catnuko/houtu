use bevy::{ecs::system::Command, prelude::*};

use crate::box3d::Box3d;
#[derive(Component)]
pub struct OrientedBoundingBox {
    pub center: Vec3,
    pub halfAxes: Mat3,
}
impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            halfAxes: Mat3::IDENTITY,
        }
    }
}
impl OrientedBoundingBox {
    pub fn fromPoints(points: &[Vec3]) -> Self {
        //实现这个功能
        let mut obb = Self::default();
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for point in points {
            min = min.min(*point);
            max = max.max(*point);
        }
        obb.center = (min + max) / 2.0;
        obb.halfAxes = Mat3::from_cols(
            (max.x - min.x) / 2.0 * Vec3::X,
            (max.y - min.y) / 2.0 * Vec3::Y,
            (max.z - min.z) / 2.0 * Vec3::Z,
        );

        obb
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
        app.add_startup_system(setup);
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
