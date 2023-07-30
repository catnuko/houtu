use std::borrow::Borrow;

use bevy::{math::DVec4, prelude::Image};

use super::{
    imagery::{Imagery, ImageryState, ShareMutImagery},
    imagery_layer::ImageryLayer,
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

    // pub fn process_state_machine(
    //     &mut self,
    //     skip_loading: bool,
    //     imagery_layer: &mut ImageryLayer,
    //     imagery_datasource: &XYZDataSource,
    //     quad_tile_rectangle: &Rectangle,
    //     asset_server: &Res<AssetServer>,
    //     images: &mut ResMut<Assets<Image>>,
    //     render_world_queue: &mut ResMut<ReprojectTextureTaskQueue>,
    //     indices_and_edges_cache: &mut IndicesAndEdgesCacheArc,
    //     render_device: &Res<RenderDevice>,
    //     globe_camera: &GlobeCamera,
    // ) -> bool {
    //     let loading_imagery = self
    //         .get_loading_imagery_mut(imagery_layer)
    //         .expect("imagery.loading_imagery");
    //     // let imagery_layer = loading_imagery.imagery_layer;

    //     loading_imagery.process_state_machine(
    //         asset_server,
    //         imagery_datasource,
    //         !self.use_web_mercator_t,
    //         skip_loading,
    //         images,
    //         render_world_queue,
    //         indices_and_edges_cache,
    //         render_device,
    //         globe_camera,
    //     );

    //     if loading_imagery.state == ImageryState::READY {
    //         if self.ready_imagery.is_some() {
    //             // self.ready_imagery.release_reference();
    //         }
    //         self.ready_imagery = self.loading_imagery;
    //         self.loading_imagery = None;
    //         self.texture_translation_and_scale =
    //             Some(imagery_layer.calculate_texture_translation_and_scale(
    //                 self,
    //                 quad_tile_rectangle,
    //                 &imagery_datasource.tiling_scheme,
    //             ));
    //         return true; // done loading
    //     }
    //     let v = if let Some(parent_key) = loading_imagery.parent.and_then(|f| Some(f.clone())) {
    //         let mut ancestor = imagery_layer.get_imagery(&parent_key);
    //         // Find some ancestor imagery we can use while self imagery is still loading.
    //         let mut closest_ancestor_that_needs_loading: Option<&Imagery> = None;
    //         while ancestor.is_some()
    //             && (ancestor.unwrap().state != ImageryState::READY
    //                 || (!self.use_web_mercator_t && ancestor.unwrap().texture.is_none()))
    //         {
    //             if ancestor.unwrap().state != ImageryState::FAILED
    //                 && ancestor.unwrap().state != ImageryState::INVALID
    //             {
    //                 // ancestor is still loading
    //                 if closest_ancestor_that_needs_loading.is_none() && ancestor.is_some() {
    //                     closest_ancestor_that_needs_loading = ancestor
    //                 }
    //             }
    //             ancestor = if let Some(v) = ancestor {
    //                 v.get_parent(imagery_layer)
    //             } else {
    //                 None
    //             };
    //         }
    //         Some((
    //             ancestor,
    //             closest_ancestor_that_needs_loading.and_then(|x| Some(x.key.clone())),
    //         ))
    //     } else {
    //         None
    //     };
    //     if v.is_none() {
    //         return false;
    //     }
    //     let (ancestor, closest_ancestor_that_needs_loading) = v.unwrap();

    //     if match (self.ready_imagery, ancestor) {
    //         (Some(a), Some(b)) => a.1 == b.key,
    //         _ => false,
    //     } {
    //         if self.ready_imagery.is_some() {
    //             // self.ready_imagery.release_reference();
    //         }

    //         self.ready_imagery = Some((imagery_layer.entity, ancestor.unwrap().key.clone()));

    //         if ancestor.is_some() {
    //             // ancestor.add_reference();
    //             self.texture_translation_and_scale =
    //                 Some(imagery_layer.calculate_texture_translation_and_scale(
    //                     self,
    //                     quad_tile_rectangle,
    //                     &imagery_datasource.tiling_scheme,
    //                 ));
    //         }
    //     }
    //     let loading_imagery = self
    //         .get_loading_imagery_mut(imagery_layer)
    //         .expect("imagery.loading_imagery");
    //     if loading_imagery.state == ImageryState::FAILED
    //         || loading_imagery.state == ImageryState::INVALID
    //     {
    //         // The imagery tile is failed or invalid, so we'd like to use an ancestor instead.
    //         if closest_ancestor_that_needs_loading.is_some() {
    //             // Push the ancestor's load process along a bit.  self is necessary because some ancestor imagery
    //             // tiles may not be attached directly to a terrain tile.  Such tiles will never load if
    //             // we don't do it here.
    //             let closest_ancestor_that_needs_loading = imagery_layer
    //                 .get_imagery_mut(closest_ancestor_that_needs_loading.as_ref().unwrap());
    //             closest_ancestor_that_needs_loading
    //                 .unwrap()
    //                 .process_state_machine(
    //                     asset_server,
    //                     imagery_datasource,
    //                     !self.use_web_mercator_t,
    //                     skip_loading,
    //                     images,
    //                     render_world_queue,
    //                     indices_and_edges_cache,
    //                     render_device,
    //                     globe_camera,
    //                 );
    //             return false; // not done loading
    //         }
    //         // self imagery tile is failed or invalid, and we have the "best available" substitute.
    //         return true; // done loading
    //     }

    //     return false; // not done loading
    // }
}
