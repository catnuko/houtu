use std::borrow::Borrow;

use bevy::{
    math::DVec4,
    prelude::{AssetServer, Assets, Image},
    render::renderer::RenderDevice,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    imagery::{Imagery, ImageryState, ShareMutImagery},
    imagery_layer::ImageryLayer,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_tile::QuadtreeTile,
    quadtree_tile_storage::QuadtreeTileStorage,
    reproject_texture::ReprojectTextureTaskQueue,
    tile_key::TileKey,
};

pub struct TileImagery {
    pub texture_coordinate_rectangle: Option<DVec4>,
    pub texture_translation_and_scale: Option<DVec4>,
    pub use_web_mercator_t: bool,
    pub loading_imagery: Option<ShareMutImagery>,
    pub ready_imagery: Option<ShareMutImagery>,
}
impl TileImagery {
    pub fn new(
        imagery: ShareMutImagery,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) -> Self {
        Self {
            texture_coordinate_rectangle,
            use_web_mercator_t,
            texture_translation_and_scale: None,
            ready_imagery: None,
            loading_imagery: Some(imagery),
        }
    }

    pub fn process_state_machine(
        tile: &mut QuadtreeTile,
        tile_imagery_index: usize,
        skip_loading: bool,
        imagery_layer: &mut ImageryLayer,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
    ) -> bool {
        let tile_imagery = tile.data.imagery.get_mut(tile_imagery_index).unwrap();
        let loading_imagery_cloned = tile_imagery.loading_imagery.as_mut().unwrap().clone();
        let mut loading_imagery = loading_imagery_cloned.lock();
        loading_imagery.process_state_machine(
            asset_server,
            !tile_imagery.use_web_mercator_t,
            skip_loading,
            images,
            render_world_queue,
            indices_and_edges_cache,
            render_device,
            globe_camera,
            &imagery_layer.imagery_provider,
        );
        // let tile = storage.get_mut(&tile_key).unwrap();
        if loading_imagery.state == ImageryState::READY {
            bevy::log::info!("imagery of tile {:?} is ready", tile.key);

            if tile_imagery.ready_imagery.is_some() {
                // tile_imagery.ready_imagery.release_reference();
            }
            tile_imagery.ready_imagery = tile_imagery.loading_imagery.clone();
            tile_imagery.loading_imagery = None;
            tile_imagery.texture_translation_and_scale = Some(
                imagery_layer
                    .calculate_texture_translation_and_scale(tile.rectangle.clone(), tile_imagery),
            );
            return true; // done loading
        }

        let mut ancestor = loading_imagery
            .parent
            .as_ref()
            .and_then(|x| Some(x.clone()));

        let mut closest_ancestor_that_needs_loading: Option<ShareMutImagery> = None;
        while ancestor.is_some()
            && ancestor.as_ref().unwrap().clone().lock().state != ImageryState::READY
            || (!tile_imagery.use_web_mercator_t
                && ancestor.as_ref().unwrap().clone().lock().texture.is_none())
        {
            let ancestor_cloned = tile_imagery.loading_imagery.as_mut().unwrap().clone();
            let ancestor_imagery = ancestor_cloned.lock();
            if ancestor_imagery.state != ImageryState::FAILED
                && ancestor_imagery.state != ImageryState::INVALID
            {
                if closest_ancestor_that_needs_loading.is_none() {
                    closest_ancestor_that_needs_loading = ancestor.clone();
                }
            }
            ancestor = ancestor_imagery
                .parent
                .as_ref()
                .and_then(|x| Some(x.clone()));
        }
        if (tile_imagery.ready_imagery.is_some()
            && ancestor.is_some()
            && tile_imagery.ready_imagery.as_ref().unwrap().lock().key
                != ancestor.as_ref().unwrap().lock().key)
            || (tile_imagery.ready_imagery.is_none() && ancestor.is_none())
        {
            if tile_imagery.ready_imagery.is_some() {
                //readsy_imagery减少引用
            }
            tile_imagery.ready_imagery = ancestor.as_ref().and_then(|x| Some(x.clone()));
            if ancestor.is_some() {
                //ancestor增加引用
                tile_imagery.texture_translation_and_scale =
                    Some(imagery_layer.calculate_texture_translation_and_scale(
                        tile.rectangle.clone(),
                        tile_imagery,
                    ));
            }
        }
        let loading_imagery = tile_imagery.loading_imagery.as_ref().unwrap().lock();
        if loading_imagery.state == ImageryState::FAILED
            || loading_imagery.state == ImageryState::INVALID
        {
            if closest_ancestor_that_needs_loading.is_some() {
                closest_ancestor_that_needs_loading
                    .as_mut()
                    .unwrap()
                    .lock()
                    .process_state_machine(
                        asset_server,
                        !tile_imagery.use_web_mercator_t,
                        skip_loading,
                        images,
                        render_world_queue,
                        indices_and_edges_cache,
                        render_device,
                        globe_camera,
                        &imagery_layer.imagery_provider,
                    );
                return false;
            }
            return true;
        }
        return false;
    }
}
