use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::shape::{Box, Cylinder};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
mod ellipsoid_shape;
pub use ellipsoid_shape::*;
use houtu_scene::Ellipsoid;
#[derive(Component)]
pub struct Shape;
pub struct GlobePlugin;
impl Default for GlobePlugin {
    fn default() -> Self {
        Self
    }
}
impl bevy::app::Plugin for GlobePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ellipsoid::WGS84);
        app.add_startup_system(setup);
    }
}
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
    let ellipsoid = Ellipsoid::WGS84;
    let x = ellipsoid.semimajor_axis() as f32;
    let y = ellipsoid.semimajor_axis() as f32;
    let z = ellipsoid.semiminor_axis() as f32;
    // let mesh: Mesh = EllipsoidShape::from_ellipsoid(ellipsoid).into();

    // commands.spawn((PbrBundle {
    //     mesh: meshes.add(mesh),
    //     material: debug_material.into(),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     // visibility: Visibility::Hidden,
    //     ..default()
    // },));
    // commands.spawn({
    //     MaterialMeshBundle {
    //         mesh: meshes.add(Box::default().into()),
    //         transform: Transform {
    //             translation: Vec3 {
    //                 x: 0.0,
    //                 y: -x - 10000.0,
    //                 z: 0.,
    //             },
    //             scale: Vec3 {
    //                 x: 120000.0,
    //                 y: 120000.0,
    //                 z: 120000.0,
    //             },
    //             ..Default::default()
    //         },
    //         material: materials.add(Color::GREEN.into()),
    //         ..Default::default()
    //     }
    // });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(x + 10000000., x + 10000000., x + 10000000.),
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
