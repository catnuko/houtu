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
    imagery_layer::{ImageryLayer, XYZDataSource},
    reproject_texture::ReprojectTextureTaskQueue,
    tile_quad_tree::IndicesAndEdgesCacheArc,
    TileKey,
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
    pub imageryLayer: Entity,
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
            imageryLayer: imagery_layer_entity,
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
    pub fn addReference(&mut self) {
        self.referenceCount += 1;
    }
    pub fn releaseReference(&mut self) {
        self.referenceCount -= 1;
    }
    pub fn createPlaceholder(imageryLayer: Entity, parent: Option<TileKey>) -> Self {
        let mut me = Self::new(
            TileKey {
                x: 0,
                y: 0,
                level: 0,
            },
            imageryLayer,
            parent,
        );
        me.state = ImageryState::PLACEHOLDER;
        me
    }
    pub fn processStateMachine(
        &mut self,
        asset_server: &Res<AssetServer>,
        imagery_datasource: &XYZDataSource,
        needGeographicProjection: bool,
        skipLoading: bool,
        images: &mut ResMut<Assets<Image>>,
        render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
        indicesAndEdgesCache: &mut IndicesAndEdgesCacheArc,
        render_device: &Res<RenderDevice>,
        globe_camera: &GlobeCamera,
    ) {
        if (self.state == ImageryState::UNLOADED && !skipLoading) {
            self.state = ImageryState::TRANSITIONING;
            let request = imagery_datasource.requestImage(&self.key, asset_server);
            if let Some(v) = request {
                self.texture = Some(v);
                self.state = ImageryState::RECEIVED;
            } else {
                self.state = ImageryState::UNLOADED;
            }
        }

        if (self.state == ImageryState::RECEIVED) {
            self.state = ImageryState::TRANSITIONING;
            self.state = ImageryState::TEXTURE_LOADED;
        }

        // If the imagery is already ready, but we need a geographic version and don't have it yet,
        // we still need to do the reprojection step. self can happen if the Web Mercator version
        // is fine initially, but the geographic one is needed later.
        let needsReprojection = self.state == ImageryState::READY && needGeographicProjection;

        if (self.state == ImageryState::TEXTURE_LOADED || needsReprojection) {
            self.state = ImageryState::TRANSITIONING;
            ImageryLayer::_reprojectTexture(
                self,
                needGeographicProjection,
                images,
                256,
                256,
                render_world_queue,
                indicesAndEdgesCache,
                render_device,
                globe_camera,
            );
        }
    }
}
#[derive(Component)]
pub struct TileImagery {
    pub textureCoordinateRectangle: Option<DVec4>,
    pub textureTranslationAndScale: Option<DVec4>,
    pub useWebMercatorT: bool,
    pub loadingImagery: Option<(Entity, TileKey)>,
    pub readyImagery: Option<(Entity, TileKey)>,
}
impl TileImagery {
    pub fn new(
        imagery_layer: Entity,
        imagery_key: TileKey,
        textureCoordinateRectangle: Option<DVec4>,
        useWebMercatorT: bool,
    ) -> Self {
        Self {
            textureCoordinateRectangle,
            useWebMercatorT,
            textureTranslationAndScale: None,
            readyImagery: None,
            loadingImagery: Some((imagery_layer, imagery_key)),
        }
    }
    pub fn get_loading_imagery_layer_entity(&self) -> Entity {
        return self.loadingImagery.unwrap().0.clone();
    }
    pub fn get_loading_imagery_mut<'a>(
        &self,
        imagery_layer: &'a mut ImageryLayer,
    ) -> Option<&'a mut Imagery> {
        self.loadingImagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery_mut(&x.1))
    }
    pub fn get_loading_imagery<'a>(&self, imagery_layer: &'a ImageryLayer) -> Option<&'a Imagery> {
        self.loadingImagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery(&x.1))
    }

    pub fn get_ready_imagery<'a>(&self, imagery_layer: &'a ImageryLayer) -> Option<&'a Imagery> {
        self.readyImagery
            .as_ref()
            .and_then(|x| imagery_layer.get_imagery(&x.1))
    }

    pub fn processStateMachine(
        &mut self,
        skipLoading: bool,
        imagery_layer: &mut ImageryLayer,
        imagery_datasource: &XYZDataSource,
        quad_tile_rectangle: &Rectangle,
        asset_server: &Res<AssetServer>,
        images: &mut ResMut<Assets<Image>>,
        render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
        indicesAndEdgesCache: &mut IndicesAndEdgesCacheArc,
        render_device: &Res<RenderDevice>,
        globe_camera: &GlobeCamera,
    ) -> bool {
        let loadingImagery = self
            .get_loading_imagery_mut(imagery_layer)
            .expect("imagery.loading_imagery");
        // let imageryLayer = loadingImagery.imageryLayer;

        loadingImagery.processStateMachine(
            asset_server,
            imagery_datasource,
            !self.useWebMercatorT,
            skipLoading,
            images,
            render_world_queue,
            indicesAndEdgesCache,
            render_device,
            globe_camera,
        );

        if (loadingImagery.state == ImageryState::READY) {
            if self.readyImagery.is_some() {
                // self.readyImagery.releaseReference();
            }
            self.readyImagery = self.loadingImagery;
            self.loadingImagery = None;
            self.textureTranslationAndScale =
                Some(imagery_layer._calculateTextureTranslationAndScale(
                    self,
                    quad_tile_rectangle,
                    &imagery_datasource.tiling_scheme,
                ));
            return true; // done loading
        }
        let v = if let Some(parent_key) = loadingImagery.parent.and_then(|f| Some(f.clone())) {
            let mut ancestor = imagery_layer.get_imagery(&parent_key);
            // Find some ancestor imagery we can use while self imagery is still loading.
            let mut closestAncestorThatNeedsLoading: Option<&Imagery> = None;
            while ancestor.is_some()
                && (ancestor.unwrap().state != ImageryState::READY
                    || (!self.useWebMercatorT && ancestor.unwrap().texture.is_none()))
            {
                if (ancestor.unwrap().state != ImageryState::FAILED
                    && ancestor.unwrap().state != ImageryState::INVALID)
                {
                    // ancestor is still loading
                    if closestAncestorThatNeedsLoading.is_none() && ancestor.is_some() {
                        closestAncestorThatNeedsLoading = ancestor
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
                closestAncestorThatNeedsLoading.and_then(|x| Some(x.key.clone())),
            ))
        } else {
            None
        };
        if v.is_none() {
            return false;
        }
        let (ancestor, closestAncestorThatNeedsLoading) = v.unwrap();

        if match (self.readyImagery, ancestor) {
            (Some(a), Some(b)) => a.1 == b.key,
            _ => false,
        } {
            if self.readyImagery.is_some() {
                // self.readyImagery.releaseReference();
            }

            self.readyImagery = Some((imagery_layer.entity, ancestor.unwrap().key.clone()));

            if ancestor.is_some() {
                // ancestor.addReference();
                self.textureTranslationAndScale =
                    Some(imagery_layer._calculateTextureTranslationAndScale(
                        self,
                        quad_tile_rectangle,
                        &imagery_datasource.tiling_scheme,
                    ));
            }
        }
        let loadingImagery = self
            .get_loading_imagery_mut(imagery_layer)
            .expect("imagery.loading_imagery");
        if (loadingImagery.state == ImageryState::FAILED
            || loadingImagery.state == ImageryState::INVALID)
        {
            // The imagery tile is failed or invalid, so we'd like to use an ancestor instead.
            if closestAncestorThatNeedsLoading.is_some() {
                // Push the ancestor's load process along a bit.  self is necessary because some ancestor imagery
                // tiles may not be attached directly to a terrain tile.  Such tiles will never load if
                // we don't do it here.
                let closestAncestorThatNeedsLoading = imagery_layer
                    .get_imagery_mut(closestAncestorThatNeedsLoading.as_ref().unwrap());
                closestAncestorThatNeedsLoading
                    .unwrap()
                    .processStateMachine(
                        asset_server,
                        imagery_datasource,
                        !self.useWebMercatorT,
                        skipLoading,
                        images,
                        render_world_queue,
                        indicesAndEdgesCache,
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
