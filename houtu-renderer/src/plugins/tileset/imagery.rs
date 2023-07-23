use std::sync::Arc;

use bevy::{
    math::DVec4,
    prelude::{AssetServer, Assets, Component, Entity, Handle, Image, Query, ResMut},
    prelude::{Res, Visibility},
    render::renderer::RenderDevice,
};
use houtu_scene::Rectangle;

use crate::plugins::camera::GlobeCamera;

use super::{
    imagery_layer::ImageryLayer, indices_and_edges_cache::IndicesAndEdgesCacheArc,
    reproject_texture::ReprojectTextureTaskQueue, xyz_datasource::XYZDataSource, TileKey,
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ImageryState {
    UNLOADED = 0,
    TRANSITIONING = 1,
    RECEIVED = 2,
    TEXTURE_LOADED = 3,
    READY = 4,
    FAILED = 5,
    INVALID = 6,
    PLACEHOLDER = 7,
}
#[derive(Component)]
pub struct Imagery {
    pub key: TileKey,
    pub state: ImageryState,
    pub imageUrl: Option<String>,
    pub texture: Option<Handle<Image>>,
    pub rectangle: Rectangle,
    pub imagery_layer: Entity,
    referenceCount: u32,
    pub parent: Option<TileKey>,
}
impl Imagery {
    pub fn id(&self) -> String {
        self.key.get_id()
    }
    pub fn new(key: TileKey, imagery_layer_entity: Entity, parent: Option<TileKey>) -> Self {
        Self {
            key: key,
            state: ImageryState::UNLOADED,
            texture: None,
            imageUrl: None,

            rectangle: Rectangle::MAX_VALUE.clone(),
            imagery_layer: imagery_layer_entity,
            referenceCount: 0,
            parent,
        }
    }
    pub fn get_parent<'a>(&self, imagery_layer: &'a ImageryLayer) -> Option<&'a Imagery> {
        self.parent
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery(x))
    }
    pub fn get_parent_mut<'a>(
        &self,
        imagery_layer: &'a mut ImageryLayer,
    ) -> Option<&'a mut Imagery> {
        self.parent
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery_mut(x))
    }
    pub fn set_texture(&mut self, new_texture: Handle<Image>) {
        self.texture = Some(new_texture);
    }
    pub fn add_reference(&mut self) {
        self.referenceCount += 1;
    }
    pub fn release_reference(&mut self) {
        self.referenceCount -= 1;
    }
    pub fn create_placeholder(imagery_layer: Entity, parent: Option<TileKey>) -> Self {
        let mut me = Self::new(
            TileKey {
                x: 0,
                y: 0,
                level: 0,
            },
            imagery_layer,
            parent,
        );
        me.state = ImageryState::PLACEHOLDER;
        me
    }
    pub fn process_state_machine(
        &mut self,
        asset_server: &Res<AssetServer>,
        imagery_datasource: &XYZDataSource,
        need_geographic_projection: bool,
        skip_loading: bool,
        images: &mut ResMut<Assets<Image>>,
        render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
        indices_and_edges_cache: &mut IndicesAndEdgesCacheArc,
        render_device: &Res<RenderDevice>,
        globe_camera: &GlobeCamera,
    ) {
        if self.state == ImageryState::UNLOADED && !skip_loading {
            self.state = ImageryState::TRANSITIONING;
            let request = imagery_datasource.requestImage(&self.key, asset_server);
            if let Some(v) = request {
                self.texture = Some(v);
                self.state = ImageryState::RECEIVED;
            } else {
                self.state = ImageryState::UNLOADED;
            }
        }

        if self.state == ImageryState::RECEIVED {
            self.state = ImageryState::TRANSITIONING;
            self.state = ImageryState::TEXTURE_LOADED;
        }

        // If the imagery is already ready, but we need a geographic version and don't have it yet,
        // we still need to do the reprojection step. self can happen if the Web Mercator version
        // is fine initially, but the geographic one is needed later.
        let needsReprojection = self.state == ImageryState::READY && need_geographic_projection;

        if self.state == ImageryState::TEXTURE_LOADED || needsReprojection {
            self.state = ImageryState::TRANSITIONING;
            ImageryLayer::reproject_texture(
                self,
                need_geographic_projection,
                images,
                256,
                256,
                render_world_queue,
                indices_and_edges_cache,
                render_device,
                globe_camera,
            );
        }
    }
}
#[derive(Component)]
pub struct TileImagery {
    pub textureCoordinateRectangle: Option<DVec4>,
    pub texture_translation_and_scale: Option<DVec4>,
    pub use_web_mercator_t: bool,
    pub loading_imagery: Option<(Entity, TileKey)>,
    pub ready_imagery: Option<(Entity, TileKey)>,
}
impl TileImagery {
    pub fn new(
        imagery_layer: Entity,
        imagery_key: TileKey,
        textureCoordinateRectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) -> Self {
        Self {
            textureCoordinateRectangle,
            use_web_mercator_t,
            texture_translation_and_scale: None,
            ready_imagery: None,
            loading_imagery: Some((imagery_layer, imagery_key)),
        }
    }
    pub fn get_loading_imagery_layer_entity(&self) -> Entity {
        return self.loading_imagery.unwrap().0.clone();
    }
    pub fn get_loading_imagery_mut<'a>(
        &self,
        imagery_layer: &'a mut ImageryLayer,
    ) -> Option<&'a mut Imagery> {
        self.loading_imagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery_mut(&x.1))
    }
    pub fn get_loading_imagery<'a>(&self, imagery_layer: &'a ImageryLayer) -> Option<&'a Imagery> {
        self.loading_imagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery(&x.1))
    }

    pub fn get_ready_imagery<'a>(&self, imagery_layer: &'a ImageryLayer) -> Option<&'a Imagery> {
        self.ready_imagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery(&x.1))
    }

    pub fn process_state_machine(
        &mut self,
        skip_loading: bool,
        imagery_layer: &mut ImageryLayer,
        imagery_datasource: &XYZDataSource,
        quad_tile_rectangle: &Rectangle,
        asset_server: &Res<AssetServer>,
        images: &mut ResMut<Assets<Image>>,
        render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
        indices_and_edges_cache: &mut IndicesAndEdgesCacheArc,
        render_device: &Res<RenderDevice>,
        globe_camera: &GlobeCamera,
    ) -> bool {
        let loading_imagery = self
            .get_loading_imagery_mut(imagery_layer)
            .expect("imagery.loading_imagery");
        // let imagery_layer = loading_imagery.imagery_layer;

        loading_imagery.process_state_machine(
            asset_server,
            imagery_datasource,
            !self.use_web_mercator_t,
            skip_loading,
            images,
            render_world_queue,
            indices_and_edges_cache,
            render_device,
            globe_camera,
        );

        if loading_imagery.state == ImageryState::READY {
            if self.ready_imagery.is_some() {
                // self.ready_imagery.release_reference();
            }
            self.ready_imagery = self.loading_imagery;
            self.loading_imagery = None;
            self.texture_translation_and_scale =
                Some(imagery_layer.calculate_texture_translation_and_scale(
                    self,
                    quad_tile_rectangle,
                    &imagery_datasource.tiling_scheme,
                ));
            return true; // done loading
        }
        let v = if let Some(parent_key) = loading_imagery.parent.and_then(|f| Some(f.clone())) {
            let mut ancestor = imagery_layer.get_imagery(&parent_key);
            // Find some ancestor imagery we can use while self imagery is still loading.
            let mut closest_ancestor_that_needs_loading: Option<&Imagery> = None;
            while ancestor.is_some()
                && (ancestor.unwrap().state != ImageryState::READY
                    || (!self.use_web_mercator_t && ancestor.unwrap().texture.is_none()))
            {
                if ancestor.unwrap().state != ImageryState::FAILED
                    && ancestor.unwrap().state != ImageryState::INVALID
                {
                    // ancestor is still loading
                    if closest_ancestor_that_needs_loading.is_none() && ancestor.is_some() {
                        closest_ancestor_that_needs_loading = ancestor
                    }
                }
                ancestor = if let Some(v) = ancestor {
                    v.get_parent(imagery_layer)
                } else {
                    None
                };
            }
            Some((
                ancestor,
                closest_ancestor_that_needs_loading.and_then(|x| Some(x.key.clone())),
            ))
        } else {
            None
        };
        if v.is_none() {
            return false;
        }
        let (ancestor, closest_ancestor_that_needs_loading) = v.unwrap();

        if match (self.ready_imagery, ancestor) {
            (Some(a), Some(b)) => a.1 == b.key,
            _ => false,
        } {
            if self.ready_imagery.is_some() {
                // self.ready_imagery.release_reference();
            }

            self.ready_imagery = Some((imagery_layer.entity, ancestor.unwrap().key.clone()));

            if ancestor.is_some() {
                // ancestor.add_reference();
                self.texture_translation_and_scale =
                    Some(imagery_layer.calculate_texture_translation_and_scale(
                        self,
                        quad_tile_rectangle,
                        &imagery_datasource.tiling_scheme,
                    ));
            }
        }
        let loading_imagery = self
            .get_loading_imagery_mut(imagery_layer)
            .expect("imagery.loading_imagery");
        if loading_imagery.state == ImageryState::FAILED
            || loading_imagery.state == ImageryState::INVALID
        {
            // The imagery tile is failed or invalid, so we'd like to use an ancestor instead.
            if closest_ancestor_that_needs_loading.is_some() {
                // Push the ancestor's load process along a bit.  self is necessary because some ancestor imagery
                // tiles may not be attached directly to a terrain tile.  Such tiles will never load if
                // we don't do it here.
                let closest_ancestor_that_needs_loading = imagery_layer
                    .get_imagery_mut(closest_ancestor_that_needs_loading.as_ref().unwrap());
                closest_ancestor_that_needs_loading
                    .unwrap()
                    .process_state_machine(
                        asset_server,
                        imagery_datasource,
                        !self.use_web_mercator_t,
                        skip_loading,
                        images,
                        render_world_queue,
                        indices_and_edges_cache,
                        render_device,
                        globe_camera,
                    );
                return false; // not done loading
            }
            // self imagery tile is failed or invalid, and we have the "best available" substitute.
            return true; // done loading
        }

        return false; // not done loading
    }
}
