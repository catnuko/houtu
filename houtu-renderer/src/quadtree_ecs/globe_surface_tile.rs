use bevy::{
    asset::LoadState,
    core::cast_slice,
    math::{DMat4, DVec4},
    prelude::{shape::Quad, *},
    render::{
        define_atomic_id,
        render_resource::{
            encase, BufferDescriptor, BufferInitDescriptor, BufferUsages, Extent3d,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
    },
    utils::{HashMap, Uuid},
};

use bevy_egui::egui::epaint::image;
use houtu_scene::{
    lerp_f32, GeographicTilingScheme, Matrix4, Rectangle, TilingScheme, WebMercatorProjection,
};

use crate::{
    camera::GlobeCamera,
    quadtree::{
        globe_surface_tile::TerrainState,
        imagery_provider::ImageryProvider,
        imagery_storage::ImageryState,
        indices_and_edges_cache::IndicesAndEdgesCacheArc,
        reproject_texture::{ParamsUniforms, ReprojectTextureTask},
        texture_minification_filter::TextureMinificationFilter,
        tile_imagery,
        tile_key::TileKey,
    },
};

use super::{
    height_map_terrain_data::HeightmapTerrainDataCom,
    imagery::{Imagery, ImageryCache},
    imagery_layer::{ImageryLayer, ImageryProviderCom},
    quadtree::Quadtree,
    quadtree_tile::{QuadtreeTile, QuadtreeTileLoadState},
    tile_imagery::{TileImagery, TileImageryVec},
};
// pub fn parent_ready_system(
//     mut quadtree_tile_query: Query<(Entity, &mut QuadtreeTile)>,
//     children_query: Query<&Children>,
// ) {
//     children_query.iter_descendants(entity)
// }
pub fn process_imagery_system(
    mut quadtree_tile_query: Query<(&mut QuadtreeTile, &mut TileImageryVec, &Rectangle)>,
    layer_query: Query<(&ImageryProviderCom, &ImageryLayer)>,
    asset_server: Res<AssetServer>,
) {
    for (mut quadtree_tile, mut tile_imagery_list, quadtree_tile_rectangle) in
        &mut quadtree_tile_query
    {
        let mut is_upsampled_only = quadtree_tile.upsampled_from_parent;
        let mut is_any_tile_loaded = false;
        let mut is_done_loading = true;
        let mut i = 0;
        let mut len = tile_imagery_list.len();
        while i < len {
            let mut tile_imagery = tile_imagery_list.0.get_mut(i).unwrap();
            if tile_imagery.loading_imagery.is_none() {
                is_upsampled_only = false;
                i += 1;
                continue;
            }

            let loading_imagery = tile_imagery.loading_imagery.as_ref().unwrap().clone();
            let (imagery_provider, imagery_layer) =
                layer_query.get(loading_imagery.get_layer_id()).unwrap();
            if loading_imagery.get_state() == ImageryState::PLACEHOLDER {
                if imagery_layer.ready && imagery_provider.0.get_ready() {
                    tile_imagery_list.remove(i);
                    len = tile_imagery_list.len();
                    i -= 1;
                    continue;
                } else {
                    is_upsampled_only = false;
                }
            }

            let this_tile_done_loading = process_tile_imagery_state_machine(
                &mut tile_imagery,
                imagery_provider,
                imagery_layer,
                false,
                &asset_server,
                quadtree_tile_rectangle,
            );
            is_done_loading = is_done_loading && this_tile_done_loading;
            is_any_tile_loaded = is_any_tile_loaded
                || this_tile_done_loading
                || tile_imagery.ready_imagery.is_some();
            is_upsampled_only = is_upsampled_only && tile_imagery.loading_imagery.is_some() && {
                let state = loading_imagery.get_state();
                state == ImageryState::FAILED || state == ImageryState::INVALID
            };

            i += 1;
        }
        quadtree_tile.upsampled_from_parent = is_upsampled_only;
        quadtree_tile.renderable =
            quadtree_tile.renderable && (is_any_tile_loaded || is_done_loading);
    }
}
pub fn process_imagery_state_machine(
    loading_imagery: Imagery,
    imagery_provider: &ImageryProviderCom,
    skip_loading: bool,
    asset_server: &AssetServer,
    need_geographic_projection: bool,
) {
    let res = loading_imagery.0.try_write();
    if res.is_err() {
        return;
    }
    let mut img = res.unwrap();
    if img.state == ImageryState::UNLOADED && !skip_loading {
        img.state = ImageryState::REQUESTING;
        let request = imagery_provider
            .0
            .request_image(&img.get_tile_key(), asset_server);
        if let Some(v) = request {
            img.texture = Some(v);
        } else {
            img.state = ImageryState::UNLOADED;
        }
    }
    if img.state == ImageryState::REQUESTING {
        let state = asset_server.get_load_state(img.texture.as_ref().unwrap());
        match state {
            Some(LoadState::Loaded) => {
                img.state = ImageryState::RECEIVED;
                // info!("imagery is ok");
            }
            Some(LoadState::Failed) => img.state = ImageryState::FAILED,
            _ => {}
        }
    }
    if img.state == ImageryState::RECEIVED {
        img.state = ImageryState::TRANSITIONING;
        img.state = ImageryState::TEXTURE_LOADED;
    }

    // If the imagery is already ready, but we need a geographic version and don't have it yet,
    // we still need to do the reprojection step. imagery can happen if the Web Mercator version
    // is fine initially, but the geographic one is needed later.
    // 如果图片已经准备好，我们需要一个geographic版本的图片。但是现在还没有，下一步将重新投影该影像
    // 投影到web墨卡托投影才算完成
    let needs_reprojection = img.state != ImageryState::READY && need_geographic_projection;

    if img.state == ImageryState::TEXTURE_LOADED || needs_reprojection {
        img.state = ImageryState::TRANSITIONING;
        // let mut key = img.key.clone();

        // ImageryLayer::reproject_texture(
        //     &mut key,
        //     imagery_storage,
        //     need_geographic_projection,
        //     images,
        //     256,
        //     256,
        //     render_world_queue,
        //     indices_and_edges_cache,
        //     render_device,
        //     globe_camera,
        //     self.imagery_provider.get_tiling_scheme().get_name(),
        // );
        img.state = ImageryState::READY;
    }
}
pub fn process_tile_imagery_state_machine(
    tile_imagery: &mut TileImagery,
    imagery_provider: &ImageryProviderCom,
    layer: &ImageryLayer,
    skip_loading: bool,
    asset_server: &AssetServer,
    quadtree_tile_rectangle: &Rectangle,
) -> bool {
    let loading_imagery = tile_imagery.loading_imagery.as_ref().unwrap().clone();
    process_imagery_state_machine(
        loading_imagery.clone(),
        imagery_provider,
        skip_loading,
        asset_server,
        !tile_imagery.use_web_mercator_t,
    );
    let img = loading_imagery.0.read().unwrap();
    if img.state == ImageryState::READY {
        tile_imagery.ready_imagery = Some(loading_imagery.clone());
        tile_imagery.loading_imagery = None;
        tile_imagery.texture_translation_and_scale =
            Some(ImageryLayer::calculate_texture_translation_and_scale(
                imagery_provider,
                quadtree_tile_rectangle.clone(),
                tile_imagery,
            ));
        return true; // done loading
    }
    let r = img.rectangle.clone();
    let mut ancestor = img.parent.clone();

    let mut closest_ancestor_that_needs_loading: Option<Imagery> = None;
    while ancestor.is_some() && {
        if let Some(ancestor_imagery) = ancestor.clone() {
            let read = ancestor_imagery.0.read().unwrap();
            read.state != ImageryState::READY
                || !tile_imagery.use_web_mercator_t && read.texture.is_none()
        } else {
            false
        }
    } {
        let t = ancestor.clone().unwrap();
        let ancestor_imagery = t.0.read().unwrap();
        // let ancestor_imagery = t.0.read().unwrap();
        if ancestor_imagery.state != ImageryState::FAILED
            && ancestor_imagery.state != ImageryState::INVALID
        {
            if closest_ancestor_that_needs_loading.is_none() {
                closest_ancestor_that_needs_loading = ancestor.clone();
            }
        }
        ancestor = ancestor_imagery.parent.clone();
    }
    if tile_imagery.ready_imagery != ancestor {
        tile_imagery.ready_imagery = ancestor.clone();
        if ancestor.is_some() {
            tile_imagery.texture_translation_and_scale =
                Some(ImageryLayer::calculate_texture_translation_and_scale(
                    imagery_provider,
                    quadtree_tile_rectangle.clone(),
                    &tile_imagery,
                ));
        }
    }
    if img.state == ImageryState::FAILED || img.state == ImageryState::INVALID {
        if closest_ancestor_that_needs_loading.is_some() {
            process_imagery_state_machine(
                closest_ancestor_that_needs_loading.clone().unwrap(),
                imagery_provider,
                skip_loading,
                asset_server,
                !tile_imagery.use_web_mercator_t,
            );
            return false;
        }
        return true;
    }
    return false;
}
pub fn process_quadtree_state_machine_system(
    mut quadtree_tile_query: Query<(
        Option<&HeightmapTerrainDataCom>,
        &mut QuadtreeTile,
        &TerrainState,
        &mut QuadtreeTileLoadState,
    )>,
) {
    for (terrain_data, mut quadtree_tile, terrain_state, mut state) in &mut quadtree_tile_query {
        let was_already_renderable = quadtree_tile.renderable;
        quadtree_tile.renderable = terrain_data.is_some_and(|x| x.0.has_mesh());
        let is_terrain_done_loading = *terrain_state == TerrainState::READY;
        quadtree_tile.upsampled_from_parent =
            terrain_data.is_some_and(|x| x.0.was_created_by_upsampling());
        //TODO process_imagery
        let is_imagery_done_loading = false;
        if is_terrain_done_loading && is_imagery_done_loading {
            *state = QuadtreeTileLoadState::DONE;
        }
        if was_already_renderable {
            quadtree_tile.renderable = true;
        }
    }
}
/// 推进terrain_stete创建HeightMapTerrainData和createMesh，upsample等,
pub fn process_terrain_state_machine_system(
    mut quadtree_tile_query: Query<(
        Entity,
        &mut QuadtreeTileLoadState,
        &mut TerrainState,
        &TileKey,
        Option<&Parent>,
    )>,
    mut commands: Commands,
    mut terrain_data_query: Query<(&mut HeightmapTerrainDataCom, &TileKey)>,
    quadtree: Res<Quadtree>,
    indices_and_edges_cache_arc: ResMut<IndicesAndEdgesCacheArc>,
) {
    for (entity, mut state, mut terrain_state, tile_key, parent) in &mut quadtree_tile_query {
        if *state != QuadtreeTileLoadState::LOADING {
            continue;
        }
        if *terrain_state == TerrainState::FAILED && parent.is_some() {
            let query_data = terrain_data_query.get(parent.unwrap().get()).unwrap();
            let terrain_data = &query_data.0 .0;
            let parent_ready = terrain_data.can_upsample();
            if parent_ready {
                //TODO process_state_machine
            }
        }
        if *terrain_state == TerrainState::FAILED {
            let query_data = terrain_data_query.get(parent.unwrap().get()).unwrap();
            let terrain_data = &query_data.0 .0;
            let parent_ready = terrain_data.can_upsample();
            if parent_ready {
                terrain_data.upsample(
                    &quadtree.tiling_scheme,
                    query_data.1.x,
                    query_data.1.y,
                    query_data.1.level,
                    tile_key.x,
                    tile_key.y,
                    tile_key.level,
                );
            }
        }
        if *terrain_state == TerrainState::UNLOADED {
            *terrain_state = TerrainState::RECEIVING;
            let terrain_data = quadtree.terrain_provider.request_tile_geometry().unwrap();
            commands
                .entity(entity)
                .insert(HeightmapTerrainDataCom(terrain_data));
            *terrain_state = TerrainState::RECEIVED;
        }
        if *terrain_state == TerrainState::RECEIVING {}
        if *terrain_state == TerrainState::RECEIVED {
            if let Ok(mut query_data) = terrain_data_query.get_mut(entity) {
                *terrain_state = TerrainState::TRANSFORMING;
                query_data.0 .0.createMesh::<GeographicTilingScheme>(
                    &quadtree.tiling_scheme,
                    tile_key.x,
                    tile_key.y,
                    tile_key.level,
                    None,
                    None,
                    indices_and_edges_cache_arc.get_cloned_cache(),
                );
                *terrain_state = TerrainState::TRANSFORMED;
            }
        }
        if *terrain_state == TerrainState::TRANSFORMING {}
        if *terrain_state == TerrainState::TRANSFORMED {
            *terrain_state = TerrainState::READY;
        }
        if *terrain_state == TerrainState::READY {}
    }
}
