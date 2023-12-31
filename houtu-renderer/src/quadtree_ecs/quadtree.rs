use bevy::{prelude::*, window::PrimaryWindow};
use houtu_scene::{
    Cartographic, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, TilingScheme,
};
use wgpu::core::command;

use crate::{
    camera::GlobeCamera,
    quadtree::{
        ellipsoid_terrain_provider::EllipsoidTerrainProvider, terrain_provider::TerrainProvider,
        tile_selection_result::TileSelectionResult, tile_key::TileKey,
    },
};

use super::{
    load::{Load, Queue, QueueType},
    quadtree_tile::{QuadtreeTile, QuadtreeTileBundle, Renderable, TileVisibility},
};
#[derive(Resource)]
pub struct Quadtree {
    pub tile_cache_size: u32,
    pub maximum_screen_space_error: f64,
    pub load_queue_time_slice: f64,
    pub loading_descendant_limit: u32,
    pub preload_ancestors: bool,
    pub preload_siblings: bool,
    pub tiles_invalidated: bool,
    pub last_tile_load_queue_length: u32,
    pub last_selection_frame_number: Option<u32>,
    pub last_frame_selection_result: TileSelectionResult,
    pub occluders: EllipsoidalOccluder,
    pub camera_position_cartographic: Option<Cartographic>,
    pub camera_reference_frame_origin_cartographic: Option<Cartographic>,
    pub terrain_provider: Box<dyn TerrainProvider>,
    pub tiling_scheme: GeographicTilingScheme,
}
impl Quadtree {
    fn new(provider: Box<dyn TerrainProvider>, tiling_scheme: GeographicTilingScheme) -> Self {
        Self {
            tile_cache_size: 100,
            loading_descendant_limit: 20,
            preload_ancestors: true,
            load_queue_time_slice: 5.0 / 1000.0,
            tiles_invalidated: false,
            maximum_screen_space_error: 2.0,
            preload_siblings: false,
            last_tile_load_queue_length: 0,
            last_selection_frame_number: None,
            last_frame_selection_result: TileSelectionResult::NONE,
            occluders: EllipsoidalOccluder::default(),
            camera_position_cartographic: None,
            camera_reference_frame_origin_cartographic: None,
            terrain_provider: provider,
            tiling_scheme: tiling_scheme,
        }
    }
}
pub fn setup_quadtree(mut commands: Commands, ellipsoid: Res<Ellipsoid>) {
    let tiling_scheme = GeographicTilingScheme::from_ellipsoid(&ellipsoid);
    let provider = Box::new(EllipsoidTerrainProvider::new());
    let quadtree = Quadtree::new(provider, tiling_scheme);
    let number_of_level_zero_tiles_x = quadtree.tiling_scheme.get_number_of_x_tiles_at_level(0);
    let number_of_level_zero_tiles_y = quadtree.tiling_scheme.get_number_of_y_tiles_at_level(0);
    for y in 0..number_of_level_zero_tiles_y {
        for x in 0..number_of_level_zero_tiles_x {
            let key = TileKey {
                x: x,
                y: y,
                level: 0,
            };
            commands.spawn(QuadtreeTileBundle::new(key));
        }
    }
    commands.insert_resource(quadtree);
}
pub fn start_render(mut quadtree_tile_query: Query<(&mut Load, &TileKey), Without<Parent>>) {
    for (mut load, _) in &mut quadtree_tile_query {
        load.queue_type = QueueType::High;
    }
}
pub fn update_tile_sse(
    mut quadtree_tile_query: Query<(&TileKey, &mut QuadtreeTile, &Load)>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut globe_camera_query: Query<&mut GlobeCamera>,
    quadtree: Res<Quadtree>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    if !globe_camera.inited {
        return;
    }
    for (tile_key, mut quadtree_tile, load) in &mut quadtree_tile_query {
        if load.queue_type == QueueType::None {
            continue;
        }
        let max_geometric_error: f64 = quadtree
            .terrain_provider
            .get_level_maximum_geometric_error(tile_key.level);

        let height = window.height() as f64;
        let sse_denominator = globe_camera.frustum.get_sse_denominator();

        let mut error = (max_geometric_error * height) / (quadtree_tile.distance * sse_denominator);

        error /= window.scale_factor();
        quadtree_tile.sse = error;
        quadtree_tile.meets_sse = error < quadtree.maximum_screen_space_error;
    }
}
pub fn visit_if_visible(
    mut root_query: Query<(&TileKey, &mut Load, &QuadtreeTile), Without<Parent>>,
    ellipsoidal_occluder: Res<EllipsoidalOccluder>,
) {
    for (tile_key, mut load, quadtree_tile) in &mut root_query {
        if !quadtree_tile.renderable {
            load.queue_type = QueueType::High;
        } else {
            let mut ancestor_meets_sse = false;
        }
    }
}

pub fn on_tile_visibility(
    mut commands: Commands,
    quadtree_tile_query: Query<(Entity, &QuadtreeTile, &TileKey, Option<&Children>)>,
    unrender_query: Query<(Entity, &QuadtreeTile), Without<Renderable>>,
) {
    for (entity, quadtree_tile, tile_key, children_option) in quadtree_tile_query.iter() {
        if quadtree_tile.visibility == TileVisibility::NONE {
            // bevy::log::info!("{:?} despawn_recursive", tile_key);
            commands.entity(entity).despawn_recursive();
        } else {
            if let None = children_option {
                commands.entity(entity).with_children(|parent| {
                    let southeast = QuadtreeTileBundle::new(tile_key.southeast());
                    let southwest = QuadtreeTileBundle::new(tile_key.southwest());
                    let northwest = QuadtreeTileBundle::new(tile_key.northwest());
                    let northeast = QuadtreeTileBundle::new(tile_key.northeast());
                    parent.spawn(southeast);
                    parent.spawn(southwest);
                    parent.spawn(northwest);
                    parent.spawn(northeast);
                    bevy::log::info!("{:?} spawn children", tile_key);
                });
            }
        }
    }
    for (entity, quadtree_tile) in unrender_query.iter() {
        if quadtree_tile.visibility != TileVisibility::NONE
            && quadtree_tile.meets_sse
            && quadtree_tile.renderable
        {
            commands.entity(entity).insert(Renderable);
        }
    }
}
