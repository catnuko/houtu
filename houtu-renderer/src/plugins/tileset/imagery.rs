use std::sync::Arc;

use bevy::{
    math::DVec4,
    prelude::{AssetServer, Component, Entity, Handle, Image, Query},
    prelude::{Res, Visibility},
};
use houtu_scene::Rectangle;

use super::{
    imagery_layer::{ImageryLayer, XYZDataSource},
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
    pub parent: Option<Arc<Imagery>>,
}
impl Imagery {
    pub fn new(key: TileKey, imagery_layer_entity: Entity, parent: Option<Arc<Imagery>>) -> Self {
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
    pub fn addReference(&mut self) {
        self.referenceCount += 1;
    }
    pub fn releaseReference(&mut self) {
        self.referenceCount -= 1;
    }
    pub fn createPlaceholder(imageryLayer: Entity, parent: Option<Arc<Imagery>>) -> Arc<Self> {
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
        Arc::new(me)
    }
    pub fn processStateMachine(
        &mut self,
        asset_server: &Res<AssetServer>,
        imagery_datasource: &XYZDataSource,
        needGeographicProjection: bool,
        skipLoading: bool,
    ) {
        if (self.state == ImageryState::UNLOADED && !skipLoading) {
            self.state = ImageryState::TRANSITIONING;
            let request = imagery_datasource.requestImage(&self.key, asset_server);
            if let Some(v) = request {
                self.texture = request;
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
            self.imageryLayer
                ._reprojectTexture(frameState, self, needGeographicProjection);
        }
    }
}
#[derive(Component)]
pub struct TileImagery {
    pub textureCoordinateRectangle: Option<DVec4>,
    pub textureTranslationAndScale: Option<DVec4>,
    pub useWebMercatorT: bool,
    pub loadingImagery: Option<Arc<Imagery>>,
    pub readyImagery: Option<Arc<Imagery>>,
}
impl TileImagery {
    pub fn new(
        imagery: Arc<Imagery>,
        textureCoordinateRectangle: Option<DVec4>,
        useWebMercatorT: bool,
    ) -> Self {
        Self {
            textureCoordinateRectangle,
            useWebMercatorT,
            textureTranslationAndScale: None,
            readyImagery: None,
            loadingImagery: Some(imagery),
        }
    }
    pub fn createPlaceholder(imageryLayer: Entity, parent: Option<Arc<Imagery>>) -> Self {
        Self::new(
            Imagery::createPlaceholder(imageryLayer, parent),
            None,
            false,
        )
    }
    pub fn processStateMachine(
        &mut self,
        skipLoading: bool,
        imagery_layer: &ImageryLayer,
        imagery_datasource: &XYZDataSource,
        quad_tile_rectangle: &Rectangle,
        asset_server: &Res<AssetServer>,
    ) -> bool {
        let loadingImagery = self.loadingImagery.as_ref().unwrap();
        // let imageryLayer = loadingImagery.imageryLayer;

        loadingImagery.processStateMachine(
            asset_server,
            imagery_datasource,
            !self.useWebMercatorT,
            skipLoading,
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

        // Find some ancestor imagery we can use while self imagery is still loading.
        let mut ancestor = loadingImagery.parent.as_ref();
        let mut closestAncestorThatNeedsLoading: Option<&Arc<Imagery>> = None;
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
            ancestor = ancestor.unwrap().parent.as_ref();
        }

        if match (self.readyImagery, ancestor) {
            (Some(a), Some(b)) => a.key == b.key,
            _ => false,
        } {
            if self.readyImagery.is_some() {
                // self.readyImagery.releaseReference();
            }

            self.readyImagery = ancestor.and_then(|x| Some(x.clone()));

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

        if (loadingImagery.state == ImageryState::FAILED
            || loadingImagery.state == ImageryState::INVALID)
        {
            // The imagery tile is failed or invalid, so we'd like to use an ancestor instead.
            if closestAncestorThatNeedsLoading.is_some() {
                // Push the ancestor's load process along a bit.  self is necessary because some ancestor imagery
                // tiles may not be attached directly to a terrain tile.  Such tiles will never load if
                // we don't do it here.
                closestAncestorThatNeedsLoading
                    .unwrap()
                    .processStateMachine(
                        asset_server,
                        imagery_datasource,
                        !self.useWebMercatorT,
                        skipLoading,
                    );
                return false; // not done loading
            }
            // self imagery tile is failed or invalid, and we have the "best available" substitute.
            return true; // done loading
        }

        return false; // not done loading
    }
}
