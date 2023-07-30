use std::cmp::Ordering;

use bevy::{core::FrameCount, prelude::Resource, window::Window};
use houtu_scene::{
    Cartographic, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Matrix4, Rectangle,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    globe_surface_tile::GlobeSurfaceTile,
    globe_surface_tile_provider::{GlobeSurfaceTileProvider, TileVisibility},
    quadtree_primitive_debug::QuadtreePrimitiveDebug,
    quadtree_tile::{Quadrant, QuadtreeTile, QuadtreeTileLoadState},
    quadtree_tile_storage::QuadtreeTileStorage,
    terrain_provider::TerrainProvider,
    tile_key::TileKey,
    tile_replacement_queue::TileReplacementQueue,
    tile_selection_result::TileSelectionResult,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails},
};

pub struct QuadtreePrimitive {
    pub tile_cache_size: u32,
    pub maximum_screen_space_error: f64,
    pub load_queue_time_slice: u32,
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
    storage: QuadtreeTileStorage,
    pub tile_load_queue_high: Vec<TileKey>,
    pub tile_load_queue_medium: Vec<TileKey>,
    pub tile_load_queue_low: Vec<TileKey>,
    pub tiles_to_render: Vec<TileKey>,
    pub tile_to_update_heights: Vec<TileKey>,
    pub tile_replacement_queue: TileReplacementQueue<'static>,
    pub tile_provider: GlobeSurfaceTileProvider,
}
pub enum QueueType {
    High,
    Medium,
    Low,
}
impl QuadtreePrimitive {
    pub fn new() -> Self {
        let mut storage = QuadtreeTileStorage::new();
        Self {
            tile_cache_size: 100,
            loading_descendant_limit: 20,
            preload_ancestors: true,
            load_queue_time_slice: 5,
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
            tile_replacement_queue: TileReplacementQueue::new(&mut storage),
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
        self.clear_tile_load_queue();
    }
    fn invalidate_all_tiles(&mut self) {
        self.tile_replacement_queue.clear();
        self.clear_tile_load_queue();
    }
    fn create_level_zero_tiles(&mut self, tiling_scheme: &GeographicTilingScheme) {
        self.storage.create_level_zero_tiles(tiling_scheme);
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
    fn queue_tile_load(&mut self, queue_type: QueueType, tile: &mut QuadtreeTile) {
        tile.load_priority = self.tile_provider.compute_tile_load_priority();
        match queue_type {
            QueueType::High => self.tile_load_queue_high.push(tile.key),
            QueueType::Medium => self.tile_load_queue_medium.push(tile.key),
            QueueType::Low => self.tile_load_queue_low.push(tile.key),
        }
    }
    pub fn render(
        &mut self,
        root_traversal_details: &mut RootTraversalDetails,
        tiling_scheme: &GeographicTilingScheme,
        globe_camera: &mut GlobeCamera,
    ) {
        if self.storage.root_len() == 0 {
            self.create_level_zero_tiles(tiling_scheme);
            let len = self.storage.root_len();
            if root_traversal_details.0.len() < len {
                root_traversal_details.0 = vec![TraversalDetails::default(); len];
            }
        }
        let occluders = &self.occluders;
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
            let tile_mut = self.storage.get_mut(key).unwrap();
            self.tile_replacement_queue.mark_tile_rendered(tile_mut.key);
            if !tile_mut.renderable {
                self.queue_tile_load(QueueType::High, tile_mut);
                self.debug.tiles_waiting_for_children += 1;
            } else {
            }
        }
    }
    pub fn endFrame() {}
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
    let tile = primitive.storage.get_mut(&tile_key).unwrap();
    if primitive.tile_provider.computeTileVisibility(
        &mut primitive.storage,
        ellipsoidal_occluder,
        globe_camera,
        tile,
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
        .mark_tile_rendered(tile_key);
    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        &tile.location,
        &tile_key,
    );
    traversal_details.all_are_renderable = true;
    traversal_details.any_were_rendered_last_frame = false;
    traversal_details.not_yet_renderable_count = 0;
    if contains_needed_position(primitive, &tile.rectangle) {
        if tile.data.vertex_array.is_none() {
            primitive.queue_tile_load(QueueType::Medium, tile);
        }
        let last_frame = primitive.last_selection_frame_number;
        let last_frame_selection_result = if tile.last_selection_result_frame == last_frame {
            tile.last_selection_result
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
    } else if primitive.preload_siblings || tile.key.level == 0 {
        // Load culled level zero tiles with low priority.
        // For all other levels, only load culled tiles if preload_siblings is enabled.
        primitive.queue_tile_load(QueueType::Low, tile);
        tile.last_selection_result = TileSelectionResult::CULLED;
    } else {
        tile.last_selection_result = TileSelectionResult::CULLED;
    }
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
    primitive.debug.tiles_visited += 1;
    primitive
        .tile_replacement_queue
        .mark_tile_rendered(tile_key);
    if tile_key.level > primitive.debug.max_depth_visited {
        primitive.debug.max_depth_visited = tile_key.level;
    }
    let tile = primitive.storage.get_mut(&tile_key).unwrap();

    let meets_sse = screen_space_error(
        &primitive.tile_provider,
        tile,
        globe_camera,
        window,
        &ellipsoidal_occluder.ellipsoid,
    ) < primitive.maximum_screen_space_error;
    let tiling_scheme = primitive.get_tiling_scheme().clone();
    primitive.storage.subdivide(&tile_key, &tiling_scheme);
    let south_west_child = primitive
        .storage
        .get_children_mut(&tile_key, Quadrant::Southwest);
    // let south_east_child = primitive
    //     .storage
    //     .get_children_mut(&tile.key, Quadrant::Southeast);
    // let north_west_child = primitive
    //     .storage
    //     .get_children_mut(&tile.key, Quadrant::Northwest);
    // let north_east_child = primitive
    //     .storage
    //     .get_children_mut(&tile.key, Quadrant::Northeast);

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
            renderable = primitive
                .tile_provider
                .can_render_without_losing_detail(tile);
        }
        if renderable {
            if meets_sse {
                primitive.queue_tile_load(QueueType::Medium, tile);
            }
            primitive.tiles_to_render.push(tile_key);

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
            primitive.queue_tile_load(QueueType::High, tile);
        }
    }
    if primitive.tile_provider.can_refine(tile) {
        let mut all_are_upsampled = {
            let mut v = south_west_child.upsampled_from_parent;
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
            primitive.queue_tile_load(QueueType::Medium, tile);
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(tile_key.southwest());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(tile_key.southeast());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(tile_key.northwest());
            primitive
                .tile_replacement_queue
                .mark_tile_rendered(tile_key.northeast());

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

        tile.last_selection_result_frame = Some(frame_count.0);
        tile.last_selection_result = TileSelectionResult::REFINED;
        let first_rendered_descendant_index = primitive.tiles_to_render.len();
        let load_index_low = primitive.tile_load_queue_low.len();
        let load_index_medium = primitive.tile_load_queue_medium.len();
        let load_index_high = primitive.tile_load_queue_high.len();
        let tiles_to_update_heights_index = primitive.tile_to_update_heights.len();
        visitVisibleChildrenNearToFar(
            primitive,
            tile_key,
            tile.location,
            traversal_details,
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
            let all_are_renderable = traversal_details.all_are_renderable;
            let any_were_rendered_last_frame = traversal_details.any_were_rendered_last_frame;
            let not_yet_renderable_count = traversal_details.not_yet_renderable_count;
            let mut queued_for_load = false;
            if !all_are_renderable && !any_were_rendered_last_frame {
                let new_len = primitive.tiles_to_render.len();
                for i in first_rendered_descendant_index..new_len {
                    let work_tile_key = primitive.tiles_to_render.get(i).unwrap();
                    let mut work_tile = primitive.storage.get_mut(&work_tile_key);
                    while work_tile.is_some() && work_tile.unwrap().key != tile_key {
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
                primitive
                    .tile_to_update_heights
                    .splice(first_rendered_descendant_index..new_len, []);
                primitive.tiles_to_render.push(tile_key);
                tile.last_selection_result = TileSelectionResult::RENDERED;
                let was_rendered_last_frame =
                    last_frame_selection_result == TileSelectionResult::RENDERED;
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
                    primitive.queue_tile_load(QueueType::Medium, tile);
                    traversal_details.not_yet_renderable_count =
                        if tile.renderable { 0 } else { 1 };
                    queued_for_load = true;
                }

                traversal_details.all_are_renderable = tile.renderable;
                traversal_details.any_were_rendered_last_frame = was_rendered_last_frame;

                if !was_rendered_last_frame {
                    // Tile is newly-rendered this frame, so update its heights.
                    // 瓦片时这帧刚渲染的，所以更新它的高度
                    primitive.tile_to_update_heights.push(tile_key);
                }
                primitive.debug.tiles_waiting_for_children += 1;
            }
            if primitive.preload_ancestors && !queued_for_load {
                primitive.queue_tile_load(QueueType::Low, tile);
            }
        }
        return;
    }
    tile.last_selection_result_frame = Some(frame_count.0);
    tile.last_selection_result = TileSelectionResult::RENDERED;

    primitive.tiles_to_render.push(tile_key);
    primitive.queue_tile_load(QueueType::High, tile);

    traversal_details.all_are_renderable = tile.renderable;
    traversal_details.any_were_rendered_last_frame =
        last_frame_selection_result == TileSelectionResult::RENDERED;
    traversal_details.not_yet_renderable_count = if tile.renderable { 0 } else { 1 };
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
    tile: &mut QuadtreeTile,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    ellipsoid: &Ellipsoid,
) -> f64 {
    let max_geometric_error: f64 = tile_provider.get_level_maximum_geometric_error(tile.key.level);

    let distance = tile.distance;
    let height = window.height() as f64;
    let sse_denominator = globe_camera.frustum.sse_denominator();

    let mut error = (max_geometric_error * height) / (distance * sse_denominator);

    error /= window.scale_factor();
    return error;
}
fn visitVisibleChildrenNearToFar(
    primitive: &mut QuadtreePrimitive,
    tile_key: TileKey,
    location: Quadrant,
    traversal_details: &mut TraversalDetails,
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
    let (east, west, south, north) = {
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
