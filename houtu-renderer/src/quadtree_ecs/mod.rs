use bevy::prelude::{Commands, IntoSystemConfigs, PreUpdate, Startup, Update};
use houtu_scene::{Ellipsoid, EllipsoidalOccluder, IndicesAndEdgesCache};
use quadtree::setup_quadtree;
use tiling_scheme::update_rectangle_system;

use tile_bounding_region::{compute_distance_to_tile, update_tile_bounding_region};

use crate::{
    quadtree::indices_and_edges_cache::IndicesAndEdgesCacheArc,
    xyz_imagery_provider::XYZImageryProvider,
};

use self::{
    globe_surface_tile::{
        process_imagery_system, process_quadtree_state_machine_system,
        process_terrain_state_machine_system,
    },
    imagery_layer::{initialize, ImageryLayerBundle},
    quadtree::{on_tile_visibility, start_render, update_tile_sse},
    tile_bounding_region::compute_tile_visibility,
};

mod globe_surface_tile;
mod height_map_terrain_data;
mod imagery;
mod imagery_layer;
mod load;
mod quadtree;
mod quadtree_tile;
mod render;
mod tile_bounding_region;
mod tile_imagery;
mod tiling_scheme;
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let ellipsoid = Ellipsoid::WGS84;
        let ellipsoidal_occluder = EllipsoidalOccluder::new(&ellipsoid);
        app.add_plugins((render::Plugin, load::Plugin))
            .insert_resource(ellipsoidal_occluder)
            .insert_resource(ellipsoid)
            .insert_resource(IndicesAndEdgesCacheArc::new())
            .add_systems(Startup, (setup_quadtree, setup_layer))
            .add_systems(PreUpdate, (start_render))
            .add_systems(
                Update,
                (
                    update_rectangle_system,
                    update_tile_bounding_region,
                    compute_distance_to_tile.after(update_tile_bounding_region),
                    compute_tile_visibility.after(compute_distance_to_tile),
                    update_tile_sse.after(compute_distance_to_tile),
                    on_tile_visibility.after(compute_tile_visibility),
                    initialize,
                    process_terrain_state_machine_system.after(initialize),
                    process_imagery_system,
                    process_quadtree_state_machine_system
                        .after(process_terrain_state_machine_system),
                ),
            );
    }
}

fn setup_layer(mut commands: Commands) {
    let provider = XYZImageryProvider {
        // url: "https://maps.omniscale.net/v2/houtuearth-4781e785/style.default/{z}/{x}/{y}.png",
        // url: "http://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        // subdomains: Some(vec!["a", "b", "c"]),
        url: "icon.png",
        // url: "https://api.maptiler.com/maps/basic-v2/256/{z}/{x}/{y}.png?key=Modv7lN1eXX1gmlqW0wY",
        ..Default::default()
    };
    let imagery_layer = ImageryLayerBundle::new(Box::new(provider));
    commands.spawn(imagery_layer);
}
