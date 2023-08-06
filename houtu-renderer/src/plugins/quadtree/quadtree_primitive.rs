use std::{cmp::Ordering};

use bevy::{
    core::FrameCount,
    prelude::{AssetServer, Assets, Image, Res, Resource},
    render::renderer::RenderDevice,
    time::Time,
    window::Window,
};
use houtu_jobs::JobSpawner;
use houtu_scene::{
    Cartographic, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Matrix4, Rectangle,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    globe_surface_tile_provider::{GlobeSurfaceTileProvider, TileVisibility},
    imagery_layer_storage::ImageryLayerStorage,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive_debug::QuadtreePrimitiveDebug,
    quadtree_tile::{Quadrant, QuadtreeTile, QuadtreeTileLoadState},
    quadtree_tile_storage::QuadtreeTileStorage,
    reproject_texture::ReprojectTextureTaskQueue,
    terrain_provider::{TerrainProvider},
    tile_key::TileKey,
    tile_replacement_queue::TileReplacementQueue,
    tile_selection_result::TileSelectionResult,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails},
};
#[derive(Resource)]
pub struct QuadtreePrimitive {
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
    pub debug: QuadtreePrimitiveDebug,
    pub storage: QuadtreeTileStorage,
    pub tile_load_queue_high: Vec<TileKey>,
    pub tile_load_queue_medium: Vec<TileKey>,
    pub tile_load_queue_low: Vec<TileKey>,
    pub tiles_to_render: Vec<TileKey>,
    pub tile_to_update_heights: Vec<TileKey>,
    pub tile_replacement_queue: TileReplacementQueue,
    pub tile_provider: GlobeSurfaceTileProvider,
}
pub enum QueueType {
    High,
    Medium,
    Low,
}
impl QuadtreePrimitive {
    pub fn new() -> Self {
        let storage = QuadtreeTileStorage::new();
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
            debug: QuadtreePrimitiveDebug::new(),
            tile_load_queue_high: vec![],
            tile_load_queue_medium: vec![],
            tile_load_queue_low: vec![],
            tiles_to_render: vec![],
            tile_to_update_heights: vec![],
            storage: storage,
            tile_replacement_queue: TileReplacementQueue::new(),
            tile_provider: GlobeSurfaceTileProvider::new(),
        }
    }
    pub fn get_tiling_scheme(&self) -> &GeographicTilingScheme {
        return self.tile_provider.get_tiling_scheme();
    }
    fn clear_tile_load_queue(&mut self) {
        self.tile_load_queue_high.clear();
        self.tile_load_queue_medium.clear();
        self.tile_load_queue_low.clear();
        self.debug.reset();
    }
    fn invalidate_all_tiles(&mut self) {
        self.tile_replacement_queue.clear();
        self.clear_tile_load_queue();
    }
    fn create_level_zero_tiles(&mut self) {
        let tiling_scheme = self.get_tiling_scheme().clone();
        self.storage.create_level_zero_tiles(&tiling_scheme);
    }
    pub fn beginFrame(&mut self) {
        if self.tiles_invalidated {
            self.invalidate_all_tiles();
            self.tiles_invalidated = false;
        }
        self.clear_tile_load_queue();
        if self.debug.suspend_lod_update {
            return;
        }
        self.tile_replacement_queue.markStartOfRenderFrame();
    }
    fn queue_tile_load(&mut self, queue_type: QueueType, tile_key: TileKey) {
        let tile = self.storage.get_mut(&tile_key).unwrap();
        tile.load_priority = self.tile_provider.compute_tile_load_priority();
        match queue_type {
            QueueType::High => self.tile_load_queue_high.push(tile.key),
            QueueType::Medium => self.tile_load_queue_medium.push(tile.key),
            QueueType::Low => self.tile_load_queue_low.push(tile.key),
        }
    }
    pub fn render(
        &mut self,
        globe_camera: &mut GlobeCamera,
        frame_count: &FrameCount,
        window: &Window,
        all_traversal_quad_details: &mut AllTraversalQuadDetails,
        root_traversal_details: &mut RootTraversalDetails,
    ) {
        if self.debug.suspend_lod_update {
            return;
        }
        self.tiles_to_render.clear();
        if self.storage.root_len() == 0 {
            self.create_level_zero_tiles();
            let len = self.storage.root_len();
            if root_traversal_details.0.len() < len {
                root_traversal_details.0 = vec![TraversalDetails::default(); len];
            }
        }
        let occluders = &mut self.occluders;
        occluders.set_camera_position(globe_camera.get_position_wc());
        let p = globe_camera.get_position_cartographic();
        let mut root_tile_list = self.storage.get_root_tile();
        root_tile_list.sort_by(|a, b| {
            let mut center = a.rectangle.center();
            let alon = center.longitude - p.longitude;
            let alat = center.latitude - p.latitude;
            center = b.rectangle.center();
            let blon = center.longitude - p.longitude;
            let blat = center.latitude - p.latitude;
            let v = alon * alon + alat * alat - (blon * blon + blat * blat);
            return if v < 0. {
                Ordering::Less
            } else if v == 0.0 {
                Ordering::Equal
            } else {
                Ordering::Greater
            };
        });
        let camera_frame_origin = globe_camera.get_transform().get_translation();
        self.camera_position_cartographic = Some(p.clone());
        self.camera_reference_frame_origin_cartographic =
            Ellipsoid::WGS84.cartesianToCartographic(&camera_frame_origin);
        for key in self.storage.root.clone().iter() {
            self.tile_replacement_queue
                .mark_tile_rendered(&mut self.storage, *key);
            let tile_mut = self.storage.get_mut(key).unwrap();
            if !tile_mut.renderable {
                let cloned = tile_mut.key.clone();
                self.queue_tile_load(QueueType::High, cloned);
                self.debug.tiles_waiting_for_children += 1;
            } else {
                let mut ancestor_meets_sse = false;
                let occluders = self.occluders.clone();
                visit_if_visible(
                    self,
                    key.clone(),
                    frame_count,
                    &occluders,
                    &mut ancestor_meets_sse,
                    globe_camera,
                    window,
                    all_traversal_quad_details,
                    root_traversal_details,
                );
            }
        }
    }
    pub fn endFrame(
        &mut self,
        frame_count: &FrameCount,
        time: &Res<Time>,
        camera: &mut GlobeCamera,
        imagery_layer_storage: &mut ImageryLayerStorage,
        job_spawner: &mut JobSpawner,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        render_device: &RenderDevice,
    ) {
        process_tile_load_queue(
            self,
            frame_count,
            time,
            camera,
            imagery_layer_storage,
            job_spawner,
            indices_and_edges_cache,
            asset_server,
            images,
            render_world_queue,
            render_device,
        );
        update_tile_load_progress_system(self);
    }
}

fn process_tile_load_queue(
    primitive: &mut QuadtreePrimitive,
    frame_count: &FrameCount,
    time: &Res<Time>,
    camera: &mut GlobeCamera,
    imagery_layer_storage: &mut ImageryLayerStorage,
    job_spawner: &mut JobSpawner,
    indices_and_edges_cache: &IndicesAndEdgesCacheArc,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    render_world_queue: &mut ReprojectTextureTaskQueue,
    render_device: &RenderDevice,
) {
    if primitive.tile_load_queue_high.len() == 0
        && primitive.tile_load_queue_medium.len() == 0
        && primitive.tile_load_queue_low.len() == 0
    {
        return;
    }

    // Remove any tiles that were not used this frame beyond the number
    // we're allowed to keep.
    let size = primitive.tile_cache_size;
    primitive
        .tile_replacement_queue
        .trimTiles(&mut primitive.storage, size);

    let end_time = time.elapsed_seconds_f64() + primitive.load_queue_time_slice;

    let mut did_some_loading = false;
    process_single_priority_load_queue(
        primitive,
        frame_count,
        &mut did_some_loading,
        end_time,
        time,
        QueueType::High,
        camera,
        imagery_layer_storage,
        job_spawner,
        indices_and_edges_cache,
        asset_server,
        images,
        render_world_queue,
        render_device,
    );
    process_single_priority_load_queue(
        primitive,
        frame_count,
        &mut did_some_loading,
        end_time,
        time,
        QueueType::Medium,
        camera,
        imagery_layer_storage,
        job_spawner,
        indices_and_edges_cache,
        asset_server,
        images,
        render_world_queue,
        render_device,
    );
    process_single_priority_load_queue(
        primitive,
        frame_count,
        &mut did_some_loading,
        end_time,
        time,
        QueueType::Low,
        camera,
        imagery_layer_storage,
        job_spawner,
        indices_and_edges_cache,
        asset_server,
        images,
        render_world_queue,
        render_device,
    );
}

fn process_single_priority_load_queue(
    primitive: &mut QuadtreePrimitive,
    _frame_count: &FrameCount,
    did_some_loading: &mut bool,
    end_time: f64,
    time: &Res<Time>,
    queue_type: QueueType,
    camera: &mut GlobeCamera,
    imagery_layer_storage: &mut ImageryLayerStorage,
    job_spawner: &mut JobSpawner,
    indices_and_edges_cache: &IndicesAndEdgesCacheArc,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    render_world_queue: &mut ReprojectTextureTaskQueue,
    render_device: &RenderDevice,
) {
    let load_queue = match queue_type {
        QueueType::High => &primitive.tile_load_queue_high,
        QueueType::Medium => &primitive.tile_load_queue_medium,
        QueueType::Low => &primitive.tile_load_queue_low,
    };
    for i in load_queue.iter() {
        primitive
            .tile_replacement_queue
            .mark_tile_rendered(&mut primitive.storage, *i);
        primitive.tile_provider.load_tile(
            &mut primitive.storage,
            &primitive.occluders,
            *i,
            imagery_layer_storage,
            job_spawner,
            indices_and_edges_cache,
            asset_server,
            images,
            render_world_queue,
            render_device,
            camera,
        );
        *did_some_loading = true;
        let seconds = time.elapsed_seconds_f64();
        if !(seconds < end_time || !*did_some_loading) {
            break;
        }
    }
}
fn visit_if_visible(
    primitive: &mut QuadtreePrimitive,
    tile_key: TileKey,
    frame_count: &FrameCount,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    ancestor_meets_sse: &mut bool,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    all_traversal_quad_details: &mut AllTraversalQuadDetails,
    root_traversal_details: &mut RootTraversalDetails,
) {
    if primitive.tile_provider.computeTileVisibility(
        &mut primitive.storage,
        ellipsoidal_occluder,
        globe_camera,
        tile_key,
    ) != TileVisibility::NONE
    {
        return visitTile(
            primitive,
            tile_key,
            globe_camera,
            ellipsoidal_occluder,
            frame_count,
            window,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
        );
    }
    primitive.debug.tiles_culled += 1;

    primitive
        .tile_replacement_queue
        .mark_tile_rendered(&mut primitive.storage, tile_key);
    let tile = primitive.storage.get_mut(&tile_key).unwrap();
    // bevy::log::info!(
    //     "{:?},{:?},{:?}",
    //     tile_key,
    //     tile.state,
    //     tile.data.terrain_state
    // );
    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        &tile.location,
        &tile_key,
    );
    traversal_details.all_are_renderable = true;
    traversal_details.any_were_rendered_last_frame = false;
    traversal_details.not_yet_renderable_count = 0;
    let rectangle = tile.rectangle.clone();
    if contains_needed_position(primitive, &rectangle) {
        let tile = primitive.storage.get_mut(&tile_key).unwrap();
        if tile.data.vertex_array.is_none() {
            primitive.queue_tile_load(QueueType::Medium, tile_key.clone());
        }
        let last_frame = primitive.last_selection_frame_number;
        let tile = primitive.storage.get_mut(&tile_key).unwrap();
        let last_frame_selection_result = if tile.last_selection_result_frame == last_frame {
            tile.last_selection_result.clone()
        } else {
            TileSelectionResult::NONE
        };
        if last_frame_selection_result != TileSelectionResult::CULLED_BUT_NEEDED
            && last_frame_selection_result != TileSelectionResult::RENDERED
        {
            // tile_quad_tree._tileToUpdateHeights.push(tile);
            primitive.tile_to_update_heights.push(tile.key);
        }
        tile.last_selection_result = TileSelectionResult::CULLED_BUT_NEEDED;
    } else if primitive.preload_siblings || tile_key.level == 0 {
        let tile = primitive.storage.get_mut(&tile_key).unwrap();
        tile.last_selection_result = TileSelectionResult::CULLED;
        primitive.queue_tile_load(QueueType::Low, tile_key.clone());
    } else {
        let tile = primitive.storage.get_mut(&tile_key).unwrap();
        tile.last_selection_result = TileSelectionResult::CULLED;
    }
    let tile = primitive.storage.get_mut(&tile_key).unwrap();
    tile.last_selection_result_frame = Some(frame_count.0);
}
fn visitTile(
    primitive: &mut QuadtreePrimitive,
    tile_key: TileKey,
    globe_camera: &mut GlobeCamera,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    frame_count: &FrameCount,
    window: &Window,
    ancestor_meets_sse: &mut bool,
    all_traversal_quad_details: &mut AllTraversalQuadDetails,
    root_traversal_details: &mut RootTraversalDetails,
) {
    let tiling_scheme = primitive.get_tiling_scheme().clone();
    primitive.storage.subdivide(&tile_key, &tiling_scheme);
    primitive.debug.tiles_visited += 1;
    primitive
        .tile_replacement_queue
        .mark_tile_rendered(&mut primitive.storage, tile_key);
    if tile_key.level > primitive.debug.max_depth_visited {
        primitive.debug.max_depth_visited = tile_key.level;
    }
    let tile = primitive.storage.get(&tile_key).unwrap();

    let meets_sse = screen_space_error(
        &primitive.tile_provider,
        tile,
        globe_camera,
        window,
        &ellipsoidal_occluder.ellipsoid,
    ) < primitive.maximum_screen_space_error;

    let last_frame = primitive.last_selection_frame_number;
    let last_frame_selection_result = if tile.last_selection_result_frame == last_frame {
        tile.last_selection_result.clone()
    } else {
        TileSelectionResult::NONE
    };
    let tile = primitive.storage.get_mut(&tile_key).unwrap();
    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        &tile.location,
        &tile_key,
    );
    if meets_sse || *ancestor_meets_sse {
        let one_rendered_last_frame =
            TileSelectionResult::originalResult(&last_frame_selection_result)
                == TileSelectionResult::RENDERED as u8;
        let two_culled_or_not_visited =
            TileSelectionResult::originalResult(&last_frame_selection_result)
                == TileSelectionResult::CULLED as u8
                || last_frame_selection_result == TileSelectionResult::NONE;
        let three_completely_loaded = tile.state == QuadtreeTileLoadState::DONE;

        let mut renderable =
            one_rendered_last_frame || two_culled_or_not_visited || three_completely_loaded;
        if !renderable {
            // renderable = primitive
            //     .tile_provider
            //     .can_render_without_losing_detail(tile,,primitive);
            renderable = false;
        }
        if renderable {
            if meets_sse {
                primitive.queue_tile_load(QueueType::Medium, tile_key.clone());
            }
            primitive.tiles_to_render.push(tile_key);

            let tile = primitive.storage.get_mut(&tile_key).unwrap();
            traversal_details.all_are_renderable = tile.renderable;
            traversal_details.any_were_rendered_last_frame =
                last_frame_selection_result == TileSelectionResult::RENDERED;
            traversal_details.not_yet_renderable_count = if tile.renderable { 0 } else { 1 };
            tile.last_selection_result_frame = Some(frame_count.0);
            tile.last_selection_result = TileSelectionResult::RENDERED;

            if !traversal_details.any_were_rendered_last_frame {
                // Tile is newly-rendered this frame, so update its heights.
                primitive.tile_to_update_heights.push(tile_key);
            }

            return;
        }
        *ancestor_meets_sse = true;

        // Load this blocker tile with high priority, but only if this tile (not just an ancestor) meets the SSE.
        if meets_sse {
            primitive.queue_tile_load(QueueType::High, tile_key.clone());
        }
    }
    let tile = primitive.storage.get_mut(&tile_key).unwrap();
    if primitive.tile_provider.can_refine(tile) {
        let all_are_upsampled = {
            let mut v = false;
            let south_west_child = primitive.storage.get(&tile_key.southwest()).unwrap();
            v = v && south_west_child.upsampled_from_parent;
            let south_east_child = primitive.storage.get(&tile_key.southeast()).unwrap();
            v = v && south_east_child.upsampled_from_parent;
            let north_west_child = primitive.storage.get(&tile_key.northwest()).unwrap();
            v = v && north_west_child.upsampled_from_parent;
            let north_east_child = primitive.storage.get(&tile_key.northeast()).unwrap();
            v = v && north_east_child.upsampled_from_parent;
            v
        };
        if all_are_upsampled {
            primitive.tiles_to_render.push(tile_key);
            primitive.queue_tile_load(QueueType::Medium, tile_key.clone());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(&mut primitive.storage, tile_key.southwest());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(&mut primitive.storage, tile_key.southeast());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(&mut primitive.storage, tile_key.northwest());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(&mut primitive.storage, tile_key.northeast());
            let tile = primitive.storage.get_mut(&tile_key).unwrap();

            traversal_details.all_are_renderable = tile.renderable;
            traversal_details.any_were_rendered_last_frame =
                last_frame_selection_result == TileSelectionResult::RENDERED;
            traversal_details.not_yet_renderable_count = if tile.renderable { 0 } else { 1 };

            tile.last_selection_result_frame = Some(frame_count.0);
            tile.last_selection_result = TileSelectionResult::RENDERED;

            if !traversal_details.any_were_rendered_last_frame {
                // Tile is newly-rendered this frame, so update its heights.
                primitive.tile_to_update_heights.push(tile_key);
            }

            return;
        }
        let tile = primitive.storage.get_mut(&tile_key).unwrap();

        tile.last_selection_result_frame = Some(frame_count.0);
        tile.last_selection_result = TileSelectionResult::REFINED;
        let first_rendered_descendant_index = primitive.tiles_to_render.len();
        let load_index_low = primitive.tile_load_queue_low.len();
        let load_index_medium = primitive.tile_load_queue_medium.len();
        let load_index_high = primitive.tile_load_queue_high.len();
        let tiles_to_update_heights_index = primitive.tile_to_update_heights.len();
        let location = tile.location.clone();
        visitVisibleChildrenNearToFar(
            primitive,
            tile_key,
            location,
            globe_camera,
            ellipsoidal_occluder,
            frame_count,
            ancestor_meets_sse,
            all_traversal_quad_details,
            root_traversal_details,
            window,
            tile_key.southwest(),
            tile_key.southeast(),
            tile_key.northwest(),
            tile_key.northeast(),
        );
        if first_rendered_descendant_index != primitive.tiles_to_render.len() {
            let traversal_details = get_traversal_details(
                all_traversal_quad_details,
                root_traversal_details,
                &location,
                &tile_key,
            );
            let all_are_renderable = traversal_details.all_are_renderable;
            let any_were_rendered_last_frame = traversal_details.any_were_rendered_last_frame;
            let not_yet_renderable_count = traversal_details.not_yet_renderable_count;
            let mut queued_for_load = false;
            if !all_are_renderable && !any_were_rendered_last_frame {
                let new_len = primitive.tiles_to_render.len();
                for i in first_rendered_descendant_index..new_len {
                    let work_tile_key = primitive.tiles_to_render.get(i).unwrap();
                    let mut work_tile = primitive.storage.get_mut(&work_tile_key);
                    while work_tile.is_some() && work_tile.as_ref().unwrap().key != tile_key {
                        let in_work_tile = work_tile.unwrap();
                        in_work_tile.last_selection_result = TileSelectionResult::from_u8(
                            TileSelectionResult::kick(&in_work_tile.last_selection_result),
                        );
                        work_tile = match in_work_tile.parent {
                            None => None,
                            Some(v) => primitive.storage.get_mut(&v),
                        };
                    }
                }
                primitive
                    .tiles_to_render
                    .splice(first_rendered_descendant_index..new_len, []);
                primitive.tile_to_update_heights.splice(
                    tiles_to_update_heights_index..primitive.tile_to_update_heights.len(),
                    [],
                );
                primitive.tiles_to_render.push(tile_key);
                let tile = primitive.storage.get_mut(&tile_key).unwrap();
                tile.last_selection_result = TileSelectionResult::RENDERED;
                let was_rendered_last_frame =
                    last_frame_selection_result == TileSelectionResult::RENDERED;
                let renderable = tile.renderable;
                if !was_rendered_last_frame
                    && not_yet_renderable_count > primitive.loading_descendant_limit
                {
                    // Remove all descendants from the load queues.
                    primitive
                        .tile_load_queue_high
                        .splice(load_index_low..new_len, []);
                    primitive
                        .tile_load_queue_medium
                        .splice(load_index_medium..new_len, []);
                    primitive
                        .tile_load_queue_low
                        .splice(load_index_high..new_len, []);
                    let renderable = tile.renderable;
                    primitive.queue_tile_load(QueueType::Medium, tile_key.clone());
                    traversal_details.not_yet_renderable_count = if renderable { 0 } else { 1 };
                    queued_for_load = true;
                }

                traversal_details.all_are_renderable = renderable;
                traversal_details.any_were_rendered_last_frame = was_rendered_last_frame;

                if !was_rendered_last_frame {
                    // Tile is newly-rendered this frame, so update its heights.
                    // 瓦片时这帧刚渲染的，所以更新它的高度
                    primitive.tile_to_update_heights.push(tile_key);
                }
                primitive.debug.tiles_waiting_for_children += 1;
            }
            if primitive.preload_ancestors && !queued_for_load {
                primitive.queue_tile_load(QueueType::Low, tile_key.clone());
            }
        }
        return;
    }
    tile.last_selection_result_frame = Some(frame_count.0);
    tile.last_selection_result = TileSelectionResult::RENDERED;

    primitive.tiles_to_render.push(tile_key);
    let renderable = tile.renderable;
    primitive.queue_tile_load(QueueType::High, tile_key.clone());

    traversal_details.all_are_renderable = renderable;
    traversal_details.any_were_rendered_last_frame =
        last_frame_selection_result == TileSelectionResult::RENDERED;
    traversal_details.not_yet_renderable_count = if renderable { 0 } else { 1 };
}
fn contains_needed_position(primitive: &mut QuadtreePrimitive, rectangle: &Rectangle) -> bool {
    return primitive.camera_position_cartographic.is_some()
        && rectangle.contains(&primitive.camera_position_cartographic.unwrap())
        || primitive
            .camera_reference_frame_origin_cartographic
            .is_some()
            && rectangle.contains(
                &primitive
                    .camera_reference_frame_origin_cartographic
                    .unwrap(),
            );
}
fn screen_space_error(
    tile_provider: &GlobeSurfaceTileProvider,
    tile: &QuadtreeTile,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    _ellipsoid: &Ellipsoid,
) -> f64 {
    let max_geometric_error: f64 = tile_provider.get_level_maximum_geometric_error(tile.key.level);

    let distance = tile.distance;
    let height = window.height() as f64;
    let sse_denominator = globe_camera.frustum.get_sse_denominator();

    let mut error = (max_geometric_error * height) / (distance * sse_denominator);

    error /= window.scale_factor();
    return error;
}
fn visitVisibleChildrenNearToFar(
    primitive: &mut QuadtreePrimitive,
    tile_key: TileKey,
    location: Quadrant,
    // traversal_details: &mut TraversalDetails,
    globe_camera: &mut GlobeCamera,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    frame_count: &FrameCount,
    ancestor_meets_sse: &mut bool,
    all_traversal_quad_details: &mut AllTraversalQuadDetails,
    root_traversal_details: &mut RootTraversalDetails,
    window: &Window,
    southwest: TileKey,
    southeast: TileKey,
    northwest: TileKey,
    northeast: TileKey,
) {
    let (east, _west, _south, north) = {
        let v = primitive.storage.get(&southwest).unwrap();
        (
            v.rectangle.east,
            v.rectangle.west,
            v.rectangle.south,
            v.rectangle.north,
        )
    };
    let camera_position = globe_camera.get_position_cartographic();
    if camera_position.longitude < east {
        if camera_position.latitude < north {
            // Camera in southwest quadrant
            visit_if_visible(
                primitive,
                southwest,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                southeast,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                northwest,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                northeast,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
        } else {
            // Camera in northwest quadrant
            visit_if_visible(
                primitive,
                northwest,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                southwest,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                northeast,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
            visit_if_visible(
                primitive,
                southeast,
                frame_count,
                ellipsoidal_occluder,
                ancestor_meets_sse,
                globe_camera,
                window,
                all_traversal_quad_details,
                root_traversal_details,
            );
        }
    } else if camera_position.latitude < north {
        // Camera southeast quadrant
        visit_if_visible(
            primitive,
            southeast,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            southwest,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            northeast,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            northwest,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
    } else {
        // Camera in northeast quadrant
        visit_if_visible(
            primitive,
            northeast,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            northwest,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            southeast,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
        visit_if_visible(
            primitive,
            southwest,
            frame_count,
            ellipsoidal_occluder,
            ancestor_meets_sse,
            globe_camera,
            window,
            all_traversal_quad_details,
            root_traversal_details,
        );
    }
    let child_quad_details = all_traversal_quad_details.get(southwest.level);
    let res = child_quad_details.combine();
    let parent_quad_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        &location,
        &tile_key,
    );
    *parent_quad_details = res;
}
pub fn get_traversal_details<'a>(
    all_traversal_quad_details: &'a mut AllTraversalQuadDetails,
    root_traversal_details: &'a mut RootTraversalDetails,
    location: &Quadrant,
    key: &TileKey,
) -> &'a mut TraversalDetails {
    return match location {
        Quadrant::Southwest => &mut all_traversal_quad_details.get_mut(key.level).southwest,
        Quadrant::Southeast => &mut all_traversal_quad_details.get_mut(key.level).southeast,
        Quadrant::Northwest => &mut all_traversal_quad_details.get_mut(key.level).northwest,
        Quadrant::Northeast => &mut all_traversal_quad_details.get_mut(key.level).northeast,
        Quadrant::Root(i) => root_traversal_details.0.get_mut(*i).unwrap(),
    };
}
pub struct TileLoadEvent(pub u32);
fn update_tile_load_progress_system(primitive: &mut QuadtreePrimitive) {
    let p0_count = primitive.tiles_to_render.len();
    let _p1_count = primitive.tile_to_update_heights.len();
    let p2_count = primitive.tile_load_queue_high.len();
    let p3_count = primitive.tile_load_queue_medium.len();
    let p4_count = primitive.tile_load_queue_low.len();
    let current_load_queue_length = (p2_count + p3_count + p4_count) as u32;
    if primitive.last_tile_load_queue_length != current_load_queue_length
        || primitive.tiles_invalidated
    {
        primitive.last_tile_load_queue_length = current_load_queue_length;
        // tile_load_event_writer.send(TileLoadEvent(current_load_queue_length));
    }
    let debug = &mut primitive.debug;
    if debug.enable_debug_output && !debug.suspend_lod_update {
        debug.max_depth = primitive
            .tiles_to_render
            .iter()
            .map(|key| key.level)
            .max()
            .unwrap_or(0);
        debug.tiles_rendered = p0_count as u32;

        if debug.tiles_visited != debug.last_tiles_visited
            || debug.tiles_rendered != debug.last_tiles_rendered
            || debug.tiles_culled != debug.last_tiles_culled
            || debug.max_depth != debug.last_max_depth
            || debug.tiles_waiting_for_children != debug.last_tiles_waiting_for_children
            || debug.max_depth_visited != debug.last_max_depth_visited
        {
            println!("Visited {}, Rendered: {}, Culled: {}, Max Depth Rendered: {}, Max Depth Visited: {}, Waiting for children: {}",debug.tiles_visited,debug.tiles_rendered,debug.tiles_culled,debug.max_depth,debug.max_depth_visited,debug.tiles_waiting_for_children);

            debug.last_tiles_visited = debug.tiles_visited;
            debug.last_tiles_rendered = debug.tiles_rendered;
            debug.last_tiles_culled = debug.tiles_culled;
            debug.last_max_depth = debug.max_depth;
            debug.last_tiles_waiting_for_children = debug.tiles_waiting_for_children;
            debug.last_max_depth_visited = debug.max_depth_visited;
        }
    }
}
