use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use geodesy::preamble::*;
use std::f32::consts::PI;

use bevy::math::{DMat3, DVec3};
mod globe;
pub use globe::Shape;
mod geographic_projection;
// mod geographic_tiling_scheme;
// mod globe_surface_tile_provider;
// mod imagery;
// mod imagery_layer;
// mod imagery_layer_collection;
// mod imagery_layer_plugin;
// mod imagery_provider;
// mod layer_id;
// mod load_file_system;
mod projection;
// mod rectangle;
// mod tile_boundingR_region;
// mod tiling_scheme;
mod ellipsoidal_occluder;
mod web_mercator_projection;
// mod wmts_imagery_layer;
// mod wmts_imagery_provider;
mod ellipsoid;
mod geometry;
mod height_map_terrain;
mod math;
mod tile;
mod tile_key;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::pbr::PbrPlugin::default());
        // app.add_plugin(houtu_camera::Plugin::default());
        // app.add_plugin(globe::GlobePlugin::default());
        app.add_plugin(houtu_events::Plugin);
        // app.add_plugin(oriented_bounding_box::OrientedBoundingBoxPlugin::default());
        // app.add_plugin(imagery_layer_plugin::ImageryLayerPlugin::default());
        // app.add_startup_system(setup);
    }
}
// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let mesh = shape::Icosphere::default().try_into().unwrap();
//     let sphere = meshes.add(mesh);
//     let points = meshes
//         .get(&sphere)
//         .unwrap()
//         .attribute(Mesh::ATTRIBUTE_POSITION)
//         .unwrap()
//         .as_float3()
//         .unwrap()
//         .iter()
//         .map(|p| Vec3::from(*p))
//         .collect::<Vec<Vec3>>();
//     // let obb = oriented_bounding_box::OrientedBoundingBox::fromPoints(points.as_slice());

//     commands.spawn((
//         PbrBundle {
//             mesh: sphere,
//             material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
//             ..Default::default()
//         },
//         obb,
//     ));
// }
