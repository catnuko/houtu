use bevy::{
    math::DVec4,
    prelude::{AssetServer, Assets, Image},
    render::renderer::RenderDevice,
};

use crate::camera::GlobeCamera;

use super::{
    imagery_layer::ImageryLayer,
    imagery_storage::{Imagery, ImageryKey, ImageryState, ImageryStorage},
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_tile::QuadtreeTile,
    reproject_texture::ReprojectTextureTaskQueue,
    tile_key::TileKey,
};

pub struct TileImagery {
    pub texture_coordinate_rectangle: Option<DVec4>,
    pub texture_translation_and_scale: Option<DVec4>,
    pub use_web_mercator_t: bool,
    pub loading_imagery: Option<Imagery>,
    pub ready_imagery: Option<Imagery>,
}
impl TileImagery {
    pub fn new(
        imagery: Imagery,
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
    pub fn free_resources(&mut self) {
        self.loading_imagery = None;
        self.ready_imagery = None;
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
        imagery_storage: &mut ImageryStorage,
    ) -> bool {
        let tile_imagery = tile.data.imagery.get_mut(tile_imagery_index).unwrap();
        imagery_layer.process_imagery_state_machine(
            tile_imagery.loading_imagery.clone().unwrap(),
            asset_server,
            !tile_imagery.use_web_mercator_t,
            skip_loading,
            images,
            render_world_queue,
            indices_and_edges_cache,
            render_device,
            globe_camera,
            imagery_storage,
        );
        let t = tile_imagery.loading_imagery.clone().unwrap();
        let loading_imagery = t.read();
        let loading_imagery_rectangle = loading_imagery.rectangle.clone();

        if loading_imagery.state == ImageryState::READY {
            tile_imagery.ready_imagery = tile_imagery.loading_imagery.clone();
            tile_imagery.loading_imagery = None;

            tile_imagery.texture_translation_and_scale =
                Some(imagery_layer.calculate_texture_translation_and_scale(
                    tile.rectangle.clone(),
                    tile_imagery,
                    loading_imagery_rectangle.clone(),
                ));
            return true; // done loading
        }

        let mut ancestor = loading_imagery.parent.clone();
        let mut closest_ancestor_that_needs_loading: Option<Imagery> = None;
        let ancestor_ready = |ancestor: Option<Imagery>| {
            ancestor.is_some_and(|x| {
                let x = x.read();
                x.state != ImageryState::READY
                    || !tile_imagery.use_web_mercator_t && x.texture.is_none()
            })
        };
        while ancestor.is_some() && ancestor_ready(ancestor.clone()) {
            let t = ancestor.clone().unwrap();
            let ancestor_imagery = t.read();
            if ancestor_imagery.state != ImageryState::FAILED
                && ancestor_imagery.state != ImageryState::INVALID
            {
                if closest_ancestor_that_needs_loading.is_none() {
                    closest_ancestor_that_needs_loading = ancestor.clone();
                }
            }

            ancestor = ancestor_imagery.parent.clone();
        }

        if (tile_imagery.ready_imagery.is_some()
            && ancestor.is_some()
            && tile_imagery.ready_imagery != ancestor)
            || (tile_imagery.ready_imagery.is_none() && ancestor.is_none())
        {
            tile_imagery.ready_imagery = ancestor.clone();
            if ancestor.is_some() {
                //ancestor增加引用
                tile_imagery.texture_translation_and_scale =
                    Some(imagery_layer.calculate_texture_translation_and_scale(
                        tile.rectangle.clone(),
                        tile_imagery,
                        loading_imagery_rectangle.clone(),
                    ));
            }
        }
        if loading_imagery.state == ImageryState::FAILED
            || loading_imagery.state == ImageryState::INVALID
        {
            if closest_ancestor_that_needs_loading.is_some() {
                imagery_layer.process_imagery_state_machine(
                    closest_ancestor_that_needs_loading.clone().unwrap(),
                    asset_server,
                    !tile_imagery.use_web_mercator_t,
                    skip_loading,
                    images,
                    render_world_queue,
                    indices_and_edges_cache,
                    render_device,
                    globe_camera,
                    imagery_storage,
                );

                return false;
            }
            return true;
        }
        return false;
    }
}
