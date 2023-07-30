use bevy::{
    math::DVec4,
    prelude::{AssetServer, Assets, Component, Entity, Handle, Image, Query, ResMut},
    prelude::{Deref, DerefMut, Res, Visibility},
    render::renderer::RenderDevice,
    utils::Uuid,
};
use bevy_egui::egui::mutex::{Mutex, MutexGuard};
use houtu_scene::Rectangle;
use std::{borrow::Borrow, cell::RefCell};
use std::{rc::Rc, sync::Arc};

use crate::plugins::camera::GlobeCamera;

use super::{imagery_layer::ImageryLayer, tile_key::TileKey};

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
#[derive(Clone, Deref, DerefMut)]
pub struct ShareMutImagery(Arc<Mutex<Imagery>>);
impl ShareMutImagery {
    pub fn new(
        key: TileKey,
        imagery_layer_id: Uuid,
        parent: Option<ShareMutImagery>,
    ) -> ShareMutImagery {
        let imagery = Imagery::new(key, imagery_layer_id, parent);
        return ShareMutImagery(Arc::new(Mutex::new(imagery)));
    }
    #[inline]
    pub fn from_imagery(imagery: Imagery) -> ShareMutImagery {
        return ShareMutImagery(Arc::new(Mutex::new(imagery)));
    }
    #[inline]
    pub fn create_placeholder(imagery_layer_id: Uuid, parent: Option<ShareMutImagery>) -> Self {
        let imagery = Imagery::create_placeholder(imagery_layer_id, parent);
        return Self::from_imagery(imagery);
    }
    #[inline]
    pub fn get_reactangle(&self) -> Rectangle {
        let v = self.as_ref().lock();
        return v.rectangle.clone();
    }
    #[inline]
    pub fn lock(&self) -> MutexGuard<Imagery> {
        return self.as_ref().lock();
    }
    #[inline]
    pub fn set_texture(&mut self, new_texture: Handle<Image>) {
        self.lock().set_texture(new_texture);
    }
    #[inline]
    pub fn set_state(&mut self, state: ImageryState) {
        self.lock().state = state;
    }
}
pub struct Imagery {
    pub key: TileKey,
    pub state: ImageryState,
    pub image_url: Option<String>,
    pub texture: Option<Handle<Image>>,
    pub rectangle: Rectangle,
    pub reference_count: u32,
    pub parent: Option<ShareMutImagery>,
    pub imagery_layer_id: Uuid,
}
impl Imagery {
    fn new(key: TileKey, imagery_layer_id: Uuid, parent: Option<ShareMutImagery>) -> Self {
        Self {
            key: key,
            state: ImageryState::UNLOADED,
            texture: None,
            image_url: None,
            rectangle: Rectangle::MAX_VALUE.clone(),
            imagery_layer_id: imagery_layer_id,
            reference_count: 0,
            parent,
        }
    }
    #[inline]
    fn get_parent<'a>(&self) -> Option<ShareMutImagery> {
        self.parent.as_ref().and_then(|x| Some(x.clone()))
    }
    #[inline]
    fn set_texture(&mut self, new_texture: Handle<Image>) {
        self.texture = Some(new_texture);
    }
    #[inline]
    fn add_reference(&mut self) {
        self.reference_count += 1;
    }
    #[inline]
    fn release_reference(&mut self) {
        self.reference_count -= 1;
    }
    #[inline]
    fn create_placeholder(imagery_layer_id: Uuid, parent: Option<ShareMutImagery>) -> Self {
        let mut me = Self::new(
            TileKey {
                x: 0,
                y: 0,
                level: 0,
            },
            imagery_layer_id,
            parent,
        );
        me.state = ImageryState::PLACEHOLDER;
        me
    }
    // pub fn process_state_machine(
    //     &mut self,
    //     asset_server: &Res<AssetServer>,
    //     imagery_datasource: &XYZDataSource,
    //     need_geographic_projection: bool,
    //     skip_loading: bool,
    //     images: &mut ResMut<Assets<Image>>,
    //     render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
    //     indices_and_edges_cache: &mut IndicesAndEdgesCacheArc,
    //     render_device: &Res<RenderDevice>,
    //     globe_camera: &GlobeCamera,
    // ) {
    //     if self.state == ImageryState::UNLOADED && !skip_loading {
    //         self.state = ImageryState::TRANSITIONING;
    //         let request = imagery_datasource.requestImage(&self.key, asset_server);
    //         if let Some(v) = request {
    //             self.texture = Some(v);
    //             self.state = ImageryState::RECEIVED;
    //         } else {
    //             self.state = ImageryState::UNLOADED;
    //         }
    //     }

    //     if self.state == ImageryState::RECEIVED {
    //         self.state = ImageryState::TRANSITIONING;
    //         self.state = ImageryState::TEXTURE_LOADED;
    //     }

    //     // If the imagery is already ready, but we need a geographic version and don't have it yet,
    //     // we still need to do the reprojection step. self can happen if the Web Mercator version
    //     // is fine initially, but the geographic one is needed later.
    //     let needsReprojection = self.state == ImageryState::READY && need_geographic_projection;

    //     if self.state == ImageryState::TEXTURE_LOADED || needsReprojection {
    //         self.state = ImageryState::TRANSITIONING;
    //         ImageryLayer::reproject_texture(
    //             self,
    //             need_geographic_projection,
    //             images,
    //             256,
    //             256,
    //             render_world_queue,
    //             indices_and_edges_cache,
    //             render_device,
    //             globe_camera,
    //         );
    //     }
    // }
}
