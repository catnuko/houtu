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
    pub loading_imagery: Option<ImageryKey>,
    pub ready_imagery: Option<ImageryKey>,
}
impl TileImagery {
    pub fn new(
        imagery_key: ImageryKey,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) -> Self {
        Self {
            texture_coordinate_rectangle,
            use_web_mercator_t,
            texture_translation_and_scale: None,
            ready_imagery: None,
            loading_imagery: Some(imagery_key),
        }
    }
    pub fn free_resources(&mut self, imagery_storage: &mut ImageryStorage) {
        if self.loading_imagery.is_some() {
            imagery_storage.release_reference(self.loading_imagery.as_ref().unwrap());
        }
        if self.ready_imagery.is_some() {
            imagery_storage.release_reference(self.ready_imagery.as_ref().unwrap());
        }
    }
    pub fn get_loading_imagery_mut<'a>(
        &self,
        imagery_storage: &'a mut ImageryStorage,
    ) -> Option<&'a mut Imagery> {
        self.loading_imagery
            .as_ref()
            .and_then(|x| imagery_storage.get_mut(x))
    }
    pub fn get_loading_imagery<'a>(
        &self,
        imagery_storage: &'a ImageryStorage,
    ) -> Option<&'a Imagery> {
        self.loading_imagery
            .as_ref()
            .and_then(|x| imagery_storage.get(x))
    }
    pub fn get_ready_imagery_mut<'a>(
        &self,
        imagery_storage: &'a mut ImageryStorage,
    ) -> Option<&'a mut Imagery> {
        self.ready_imagery
            .as_ref()
            .and_then(|x| imagery_storage.get_mut(x))
    }
    pub fn get_ready_imagery<'a>(
        &self,
        imagery_storage: &'a ImageryStorage,
    ) -> Option<&'a Imagery> {
        self.ready_imagery
            .as_ref()
            .and_then(|x| imagery_storage.get(x))
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
            tile_imagery.loading_imagery.as_ref().unwrap(),
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
        let loading_imagery = imagery_storage
            .get_mut(tile_imagery.loading_imagery.as_ref().unwrap())
            .unwrap();
        if loading_imagery.state == ImageryState::READY {
            // bevy::log::info!("imagery of tile {:?} is ready", tile.key);

            if tile_imagery.ready_imagery.is_some() {
                imagery_storage.release_reference(tile_imagery.ready_imagery.as_ref().unwrap());
            }
            tile_imagery.ready_imagery = tile_imagery.loading_imagery.clone();
            tile_imagery.loading_imagery = None;

            // bevy::log::info!(
            //     "{:?},{:?}",
            //     tile_imagery.key,
            //     tile_imagery.ready_imagery.as_ref().unwrap()
            // );
            let loading_imagery = imagery_storage
                .get_mut(tile_imagery.ready_imagery.as_ref().unwrap())
                .unwrap();
            let r = loading_imagery.rectangle.clone();
            tile_imagery.texture_translation_and_scale =
                Some(imagery_layer.calculate_texture_translation_and_scale(
                    tile.rectangle.clone(),
                    tile_imagery,
                    r,
                ));
            return true; // done loading
        }
        let r = loading_imagery.rectangle.clone();
        let mut ancestor = loading_imagery
            .parent
            .as_ref()
            .and_then(|x| Some(x.clone()));

        let mut closest_ancestor_that_needs_loading: Option<ImageryKey> = None;
        while ancestor.is_some() && {
            if let Some(ancestor_imagery) = imagery_storage.get(ancestor.as_ref().unwrap()) {
                ancestor_imagery.state != ImageryState::READY
                    || !tile_imagery.use_web_mercator_t && ancestor_imagery.texture.is_none()
            } else {
                false
            }
        } {
            let ancestor_imagery = imagery_storage.get(ancestor.as_ref().unwrap()).unwrap();
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
            && tile_imagery.ready_imagery != ancestor)
            || (tile_imagery.ready_imagery.is_none() && ancestor.is_none())
        {
            if tile_imagery.ready_imagery.is_some() {
                //readsy_imagery减少引用
                imagery_storage.release_reference(tile_imagery.ready_imagery.as_ref().unwrap());
            }
            tile_imagery.ready_imagery = ancestor.as_ref().and_then(|x| Some(x.clone()));
            if ancestor.is_some() {
                //ancestor增加引用
                imagery_storage.add_reference(ancestor.as_ref().unwrap());
                tile_imagery.texture_translation_and_scale =
                    Some(imagery_layer.calculate_texture_translation_and_scale(
                        tile.rectangle.clone(),
                        tile_imagery,
                        r,
                    ));
            }
        }

        let loading_imagery = imagery_storage
            .get_mut(tile_imagery.loading_imagery.as_ref().unwrap())
            .unwrap();
        if loading_imagery.state == ImageryState::FAILED
            || loading_imagery.state == ImageryState::INVALID
        {
            if closest_ancestor_that_needs_loading.is_some() {
                imagery_layer.process_imagery_state_machine(
                    closest_ancestor_that_needs_loading.as_ref().unwrap(),
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
