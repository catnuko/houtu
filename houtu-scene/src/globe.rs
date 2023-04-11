use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use geodesy::preamble::*;
use std::f32::consts::PI;

use crate::ellipsoid::EllipsoidShape;
#[derive(Component)]
pub struct Shape;

pub fn new_ellipsoid() -> Ellipsoid {
    return Ellipsoid::named("wgs84").unwrap();
}
pub struct GlobePlugin {
    // material:Handle<StandardMaterial>,
}
impl Default for GlobePlugin {
    fn default() -> Self {
        Self {}
    }
}
impl bevy::app::Plugin for GlobePlugin {
    fn build(&self, app: &mut App) {
        // app.add_system_set(systems::system_set());
        app.add_startup_system(setup);
    }
}
const X_EXTENT: f32 = 14.5;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    let ellipsoid = Ellipsoid::named("WGS84").unwrap();
    let x = ellipsoid.semimajor_axis() as f32;
    let y = ellipsoid.semiminor_axis() as f32;
    let z = ellipsoid.semiminor_axis() as f32;
    commands.spawn((
        PbrBundle {
            mesh:  meshes.add(EllipsoidShape::from_ellipsoid(ellipsoid).into()),
            material:  materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Shape,
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(x+1000., x+1000., x+1000.),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(x+10000000., x, x).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}
/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
