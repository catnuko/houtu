use std::cmp::Ordering;
use std::f64::consts::PI;
use std::sync::Arc;

use bevy::core::FrameCount;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use houtu_scene::{
    Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Matrix4,
    Rectangle, TileBoundingRegion, TilingScheme,
};

use crate::plugins::camera::GlobeCamera;

use super::globe_surface_tile::{computeTileVisibility, GlobeSurfaceTile, TileVisibility};
use super::tile_selection_result::TileSelectionResult;
use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileData, QuadtreeTileLoadState,
    QuadtreeTileMark, QuadtreeTileOtherState, QuadtreeTileParent, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileToRender, TileToUpdateHeight,
};
use super::tile_datasource::{self, QuadTreeTileDatasourceMark, Ready, TilingSchemeWrap};
use super::tile_replacement_queue::{TileReplacementQueue, TileReplacementState};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(tile_datasource::Plugin);
        app.insert_resource(TileQuadTree::new());
        app.insert_resource(AllTraversalQuadDetails::new());
        app.insert_resource(RootTraversalDetails::new());
        app.add_system(begin_frame.before(render).before(end_frame));
    }
}
#[derive(Resource, Debug)]
pub struct TileQuadTree {
    tileCacheSize: f64,
    maximumScreenSpaceError: f64,
    _loadQueueTimeSlice: f64,
    loadingDescendantLimit: u32,
    preloadAncestors: bool,
    preloadSiblings: bool,
    _tilesInvalidated: bool,
    _lastTileLoadQueueLength: u32,
    _lastSelectionFrameNumber: Option<u32>,
    _occluders: EllipsoidalOccluder,
    _cameraPositionCartographic: Option<Cartographic>,
    _cameraReferenceFrameOriginCartographic: Option<Cartographic>,
    replacement_queue: TileReplacementQueue,
}

impl TileQuadTree {
    pub fn new() -> Self {
        Self {
            tileCacheSize: 100.,
            loadingDescendantLimit: 20,
            preloadAncestors: true,
            _loadQueueTimeSlice: 5.0,
            _tilesInvalidated: false,
            maximumScreenSpaceError: 2.0,
            preloadSiblings: false,
            _lastTileLoadQueueLength: 0,
            _lastSelectionFrameNumber: None,
            _occluders: EllipsoidalOccluder::default(),
            _cameraPositionCartographic: None,
            _cameraReferenceFrameOriginCartographic: None,
            replacement_queue: TileReplacementQueue::new(),
        }
    }
    /// 调用后将清空所有瓦片重新创建
    pub fn invalidateAllTiles(&mut self) {
        self._tilesInvalidated = true;
    }
    pub fn real_invalidateAllTiles(&mut self) {}
}
fn begin_frame(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    high_queue_query: Query<Entity, With<TileLoadHigh>>,
    medium_queue_query: Query<Entity, With<TileLoadMedium>>,
    low_queue_query: Query<Entity, With<TileLoadLow>>,
    render_queue_query: Query<Entity, With<TileToRender>>,
) {
    // 帧开始
    if (tile_quad_tree._tilesInvalidated) {
        tile_quad_tree.real_invalidateAllTiles();
        tile_quad_tree._tilesInvalidated = false;
    }
    // TODO 初始化tileProvider

    //清空队列
    high_queue_query.iter().for_each(|entity| {
        if let Some(mut entity_mut) = commands.get_entity(entity) {
            entity_mut.remove::<TileLoadHigh>();
        }
    });
    medium_queue_query.iter().for_each(|entity| {
        if let Some(mut entity_mut) = commands.get_entity(entity) {
            entity_mut.remove::<TileLoadMedium>();
        }
    });
    low_queue_query.iter().for_each(|entity| {
        if let Some(mut entity_mut) = commands.get_entity(entity) {
            entity_mut.remove::<TileLoadLow>();
        }
    });
    tile_quad_tree.replacement_queue.markStartOfRenderFrame();

    // TODO createRenderCommandsForSelectedTiles函数开始
}
fn render(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    render_queue_query: Query<Entity, With<TileToRender>>,
    mut datasource_query: Query<
        (&Ready, &TilingSchemeWrap<GeographicTilingScheme>),
        With<QuadTreeTileDatasourceMark>,
    >,
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery>,
    mut quadtree_tile_query2: Query<GlobeSurfaceTileQuery>,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
    ellipsoidalOccluder: Res<EllipsoidalOccluder>,
    mut root_traversal_details: ResMut<RootTraversalDetails>,
    frame_count: Res<FrameCount>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut all_traversal_quad_details: ResMut<AllTraversalQuadDetails>,
    mut queue_params_set: ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
        &World,
    )>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    //selectTilesForRendering
    //清空渲染列表
    render_queue_query.iter().for_each(|entity: Entity| {
        let mut entity_mut = commands.get_entity(entity).expect("entity不存在");
        entity_mut.remove::<TileToRender>();
    });
    if datasource_query.iter().len() != 1 {
        return;
    }
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    let datasource = datasource_query
        .get_single()
        .expect("QuadTreeTileDatasourceMark不存在");
    //创建根节点
    if quadtree_tile_query
        .iter_mut()
        .filter(|v| {
            if let Quadrant::Root(_) = *v.11 {
                return true;
            } else {
                return false;
            }
        })
        .count()
        == 0
    {
        let (ready, tiling_scheme_wrap) = datasource;
        if ready.0 {
            let tiling_scheme = &tiling_scheme_wrap.0;
            let numberOfLevelZeroTilesX = tiling_scheme.get_number_of_x_tiles_at_level(0);
            let numberOfLevelZeroTilesY = tiling_scheme.get_number_of_y_tiles_at_level(0);
            let mut i = 0;
            for y in 0..numberOfLevelZeroTilesY {
                for x in 0..numberOfLevelZeroTilesX {
                    let r = tiling_scheme_wrap.0.tile_x_y_to_rectange(x, y, 0);
                    make_new_quadtree_tile(
                        &mut commands,
                        TileKey {
                            x: x,
                            y: y,
                            level: 0,
                        },
                        r,
                        Quadrant::Root(i),
                        QuadtreeTileParent(TileNode::None),
                    );
                    i += 1;
                }
            }
            let count = render_queue_query.iter().count();
            if root_traversal_details.0.len() < count {
                root_traversal_details.0 = vec![TraversalDetails::default(); count]
            }
        } else {
            return;
        }
    }
    let occluders = if quadtree_tile_query.iter().count() > 1 {
        Some(EllipsoidalOccluder::default())
    } else {
        None
    };
    //按相机位置排序，从近到远
    let p = globe_camera.get_position_cartographic();
    let mut tt = vec![];
    quadtree_tile_query.iter().for_each(|x| tt.push(x));
    tt.sort_by(|a, b| {
        let mut center = a.2.center();
        let alon = center.longitude - p.longitude;
        let alat = center.latitude - p.latitude;
        center = b.2.center();
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
    //设置当前位置
    let cameraFrameOrigin = globe_camera.get_transform().get_translation();
    tile_quad_tree._cameraPositionCartographic = Some(p.clone());
    tile_quad_tree._cameraReferenceFrameOriginCartographic =
        Ellipsoid::WGS84.cartesianToCartographic(&cameraFrameOrigin);
    tt.iter().enumerate().for_each(|(_, x)| {
        let (entity, _, _, _, other_state, _, _, _, _, _, _, _, _) = x;
        tile_quad_tree
            .replacement_queue
            .markTileRendered(&mut quadtree_tile_query2, *entity);
        if !other_state.renderable {
            commands.entity(*entity).insert(TileLoadHigh);
        } else {
            let cl = { globe_camera.get_culling_volume().clone() };
            visitIfVisible(
                &mut commands,
                &mut tile_quad_tree,
                &ellipsoidalOccluder.ellipsoid,
                &ellipsoidalOccluder,
                &mut quadtree_tile_query2,
                &cl,
                &frame_count,
                &mut globe_camera,
                window,
                &datasource.1 .0,
                false,
                &mut all_traversal_quad_details,
                &mut root_traversal_details,
                &mut queue_params_set,
                *entity,
            );
        }
    });
}
fn get_traversal_details<'a>(
    all_traversal_quad_details: &'a mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &'a mut ResMut<RootTraversalDetails>,
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
fn visitTile(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    culling_volume: &CullingVolume,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    tiling_scheme: &GeographicTilingScheme,
    ancestorMeetsSse: bool,
    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
        &World,
    )>,
    quadtree_tile_entity: Entity,
) {
    let mut ancestorMeetsSse = ancestorMeetsSse;
    tile_quad_tree
        .replacement_queue
        .markTileRendered(quadtree_tile_query, quadtree_tile_entity);
    let (
        entity,
        globe_surface_tile,
        rectangle,
        parent,
        mut other_state,
        mut replacement_state,
        data,
        key,
        node_id,
        mut node_children,
        state,
        location,
        _,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let traversalDetails = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    let mut entity_mut = commands.entity(entity);

    let meetsSse = screenSpaceError(
        key,
        &mut other_state,
        globe_camera,
        window,
        ellipsoid,
        tiling_scheme,
    ) < tile_quad_tree.maximumScreenSpaceError;
    subdivide(
        entity_mut.commands(),
        node_id,
        key,
        &mut node_children,
        tiling_scheme,
    );
    let southwestChild = node_children.southwest;
    let southeastChild = node_children.southeast;
    let northwestChild = node_children.northwest;
    let northeastChild = node_children.northeast;

    let lastFrame = tile_quad_tree._lastSelectionFrameNumber;
    let lastFrameSelectionResult = if other_state._lastSelectionResultFrame == lastFrame {
        other_state._lastSelectionResult.clone()
    } else {
        TileSelectionResult::NONE
    };
    if meetsSse || ancestorMeetsSse {
        // This tile (or an ancestor) is the one we want to render this frame, but we'll do different things depending
        // on the state of this tile and on what we did _last_ frame.

        // We can render it if _any_ of the following are true:
        // 1. We rendered it (or kicked it) last frame.
        // 2. This tile was culled last frame, or it wasn't even visited because an ancestor was culled.
        // 3. The tile is completely done loading.
        // 4. a) Terrain is ready, and
        //    b) All necessary imagery is ready. Necessary imagery is imagery that was rendered with this tile
        //       or any descendants last frame. Such imagery is required because rendering this tile without
        //       it would cause detail to disappear.
        //
        // Determining condition 4 is more expensive, so we check the others first.
        //
        // Note that even if we decide to render a tile here, it may later get "kicked" in favor of an ancestor.

        let oneRenderedLastFrame = TileSelectionResult::originalResult(&lastFrameSelectionResult)
            == TileSelectionResult::RENDERED as u8;
        let twoCulledOrNotVisited = TileSelectionResult::originalResult(&lastFrameSelectionResult)
            == TileSelectionResult::CULLED as u8
            || lastFrameSelectionResult == TileSelectionResult::NONE;
        let threeCompletelyLoaded = *state == QuadtreeTileLoadState::DONE;

        let mut renderable = oneRenderedLastFrame || twoCulledOrNotVisited || threeCompletelyLoaded;

        if (!renderable) {
            // Check the more expensive condition 4 above. This requires details of the thing
            // we're rendering (e.g. the globe surface), so delegate it to the tile provider.
            renderable = false
        }

        if (renderable) {
            // Only load this tile if it (not just an ancestor) meets the SSE.
            if (meetsSse) {
                entity_mut.insert(TileLoadMedium);
            }
            entity_mut.insert(TileToRender);

            traversalDetails.allAreRenderable = other_state.renderable;
            traversalDetails.anyWereRenderedLastFrame =
                lastFrameSelectionResult == TileSelectionResult::RENDERED;
            traversalDetails.notYetRenderableCount = if other_state.renderable { 0 } else { 1 };

            other_state._lastSelectionResultFrame = Some(frame_count.0);
            other_state._lastSelectionResult = TileSelectionResult::RENDERED;

            if (!traversalDetails.anyWereRenderedLastFrame) {
                // Tile is newly-rendered this frame, so update its heights.
                entity_mut.insert(TileToUpdateHeight);
            }

            return;
        }

        // Otherwise, we can't render this tile (or its fill) because doing so would cause detail to disappear
        // that was visible last frame. Instead, keep rendering any still-visible descendants that were rendered
        // last frame and render fills for newly-visible descendants. E.g. if we were rendering level 15 last
        // frame but this frame we want level 14 and the closest renderable level <= 14 is 0, rendering level
        // zero would be pretty jarring so instead we keep rendering level 15 even though its SSE is better
        // than required. So fall through to continue traversal...
        ancestorMeetsSse = true;

        // Load this blocker tile with high priority, but only if this tile (not just an ancestor) meets the SSE.
        if (meetsSse) {
            entity_mut.insert(TileLoadHigh);
        }
    }
    //TODO canRefine
    if globe_surface_tile.terrainData.is_some() {
        let mut allAreUpsampled = true;
        if let TileNode::Internal(v) = southwestChild {
            let state = queue_params_set
                .p5()
                .get::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }
        if !allAreUpsampled {
            return;
        }
        if let TileNode::Internal(v) = southeastChild {
            let state = queue_params_set
                .p5()
                .get::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }
        if !allAreUpsampled {
            return;
        }
        if let TileNode::Internal(v) = northwestChild {
            let state = queue_params_set
                .p5()
                .get::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }
        if !allAreUpsampled {
            return;
        }
        if let TileNode::Internal(v) = northeastChild {
            let state = queue_params_set
                .p5()
                .get::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }

        if (allAreUpsampled) {
            // No point in rendering the children because they're all upsampled.  Render this tile instead.
            entity_mut.insert(TileToRender);

            // Rendered tile that's not waiting on children loads with medium priority.
            entity_mut.insert(TileLoadHigh);

            // Make sure we don't unload the children and forget they're upsampled.

            let mut markTileRendered_child =
                |tile_quad_tree: &mut ResMut<TileQuadTree>,
                 node_id: &TileNode,
                 quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>| {
                    if let TileNode::Internal(v) = node_id {
                        tile_quad_tree
                            .replacement_queue
                            .markTileRendered(quadtree_tile_query, v.clone());
                    }
                };
            markTileRendered_child(tile_quad_tree, &southwestChild, quadtree_tile_query);
            markTileRendered_child(tile_quad_tree, &southeastChild, quadtree_tile_query);
            markTileRendered_child(tile_quad_tree, &northwestChild, quadtree_tile_query);
            markTileRendered_child(tile_quad_tree, &northeastChild, quadtree_tile_query);
            let mut other_state = quadtree_tile_query
                .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
                .unwrap();
            traversalDetails.allAreRenderable = other_state.renderable;
            traversalDetails.anyWereRenderedLastFrame =
                lastFrameSelectionResult == TileSelectionResult::RENDERED;
            traversalDetails.notYetRenderableCount = if other_state.renderable { 0 } else { 1 };

            other_state._lastSelectionResultFrame = Some(frame_count.0);
            other_state._lastSelectionResult = TileSelectionResult::RENDERED;

            if (!traversalDetails.anyWereRenderedLastFrame) {
                // Tile is newly-rendered this frame, so update its heights.
                entity_mut.insert(TileToUpdateHeight);
            }

            return;
        }

        // SSE is not good enough, so refine.
        other_state._lastSelectionResultFrame = Some(frame_count.0);
        other_state._lastSelectionResult = TileSelectionResult::REFINED;

        let firstRenderedDescendantIndex = queue_params_set.p0().iter().count();
        let loadIndexLow = queue_params_set.p4().iter().count();
        let loadIndexMedium = queue_params_set.p3().iter().count();
        let loadIndexHigh = queue_params_set.p2().iter().count();
        let tilesToUpdateHeightsIndex = queue_params_set.p1().iter().count();

        // No need to add the children to the load queue because they'll be added (if necessary) when they're visited.
        visitVisibleChildrenNearToFar(
            entity_mut.commands(),
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            queue_params_set,
            all_traversal_quad_details,
            root_traversal_details,
            &southwestChild,
            &southeastChild,
            &northwestChild,
            &northeastChild,
            quadtree_tile_entity,
        );
        let key = quadtree_tile_query
            .get_component::<TileKey>(quadtree_tile_entity)
            .unwrap();
        let location = quadtree_tile_query
            .get_component::<Quadrant>(quadtree_tile_entity)
            .unwrap();
        let traversalDetails = get_traversal_details(
            all_traversal_quad_details,
            root_traversal_details,
            location,
            key,
        );
        let render_count = queue_params_set.p0().iter().count();
        // If no descendant tiles were added to the render list by the function above, it means they were all
        // culled even though this tile was deemed visible. That's pretty common.

        if (firstRenderedDescendantIndex != render_count) {
            // At least one descendant tile was added to the render list.
            // The traversalDetails tell us what happened while visiting the children.

            let allAreRenderable = traversalDetails.allAreRenderable;
            let anyWereRenderedLastFrame = traversalDetails.anyWereRenderedLastFrame;
            let notYetRenderableCount = traversalDetails.notYetRenderableCount;
            let mut queuedForLoad = false;

            if (!allAreRenderable && !anyWereRenderedLastFrame) {
                // Some of our descendants aren't ready to render yet, and none were rendered last frame,
                // so kick them all out of the render list and render this tile instead. Continue to load them though!

                // Mark the rendered descendants and their ancestors - up to this tile - as kicked.
                queue_params_set.p0().iter().enumerate().for_each(|(i, e)| {
                    if i >= firstRenderedDescendantIndex {
                        let mut workTile = e.clone();
                        while (workTile != entity) {
                            let mut work_tile = quadtree_tile_query.get_mut(workTile).unwrap();
                            let other_state = &mut work_tile.4;
                            other_state._lastSelectionResult = TileSelectionResult::from_u8(
                                TileSelectionResult::kick(&other_state._lastSelectionResult),
                            );
                            let parent = &work_tile.12;
                            if let QuadtreeTileParent(TileNode::Internal(v)) = parent {
                                workTile = v.clone();
                            }
                        }
                    }
                });

                // Remove all descendants from the render list and add this tile.
                remove_component(
                    entity_mut.commands(),
                    &queue_params_set.p0(),
                    firstRenderedDescendantIndex,
                );
                remove_component(
                    entity_mut.commands(),
                    &queue_params_set.p1(),
                    tilesToUpdateHeightsIndex,
                );
                entity_mut.insert(TileToRender);

                let mut other_state = quadtree_tile_query
                    .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
                    .unwrap();
                other_state._lastSelectionResult = TileSelectionResult::RENDERED;

                // If we're waiting on heaps of descendants, the above will take too long. So in that case,
                // load this tile INSTEAD of loading any of the descendants, and tell the up-level we're only waiting
                // on this tile. Keep doing this until we actually manage to render this tile.
                let wasRenderedLastFrame =
                    lastFrameSelectionResult == TileSelectionResult::RENDERED;
                if (!wasRenderedLastFrame
                    && notYetRenderableCount > tile_quad_tree.loadingDescendantLimit)
                {
                    // Remove all descendants from the load queues.
                    remove_component(entity_mut.commands(), &queue_params_set.p4(), loadIndexLow);
                    remove_component(
                        entity_mut.commands(),
                        &queue_params_set.p3(),
                        loadIndexMedium,
                    );
                    remove_component(entity_mut.commands(), &queue_params_set.p2(), loadIndexHigh);
                    entity_mut.insert(TileLoadMedium);
                    traversalDetails.notYetRenderableCount =
                        if other_state.renderable { 0 } else { 1 };
                    queuedForLoad = true;
                }

                traversalDetails.allAreRenderable = other_state.renderable;
                traversalDetails.anyWereRenderedLastFrame = wasRenderedLastFrame;

                if (!wasRenderedLastFrame) {
                    // Tile is newly-rendered this frame, so update its heights.
                    entity_mut.insert(TileToUpdateHeight);
                }
            }

            if (tile_quad_tree.preloadAncestors && !queuedForLoad) {
                entity_mut.insert(TileLoadLow);
            }
        }

        return;
    }
    let renderable = {
        let mut other_state = quadtree_tile_query
            .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
            .unwrap();
        other_state._lastSelectionResultFrame = Some(frame_count.0);
        other_state._lastSelectionResult = TileSelectionResult::RENDERED;
        other_state.renderable
    };
    // We'd like to refine but can't because we have no availability data for this tile's children,
    // so we have no idea if refinining would involve a load or an upsample. We'll have to finish
    // loading this tile first in order to find that out, so load this refinement blocker with
    // high priority.
    entity_mut.insert(TileToRender);
    entity_mut.insert(TileLoadHigh);
    traversalDetails.allAreRenderable = renderable;
    traversalDetails.anyWereRenderedLastFrame =
        lastFrameSelectionResult == TileSelectionResult::RENDERED;
    traversalDetails.notYetRenderableCount = if renderable { 0 } else { 1 };
}
fn visitIfVisible(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    culling_volume: &CullingVolume,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    tiling_scheme: &GeographicTilingScheme,
    ancestorMeetsSse: bool,
    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
        &World,
    )>,
    quadtree_tile_entity: Entity,
) {
    if computeTileVisibility(
        commands,
        ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
        globe_camera,
        culling_volume,
        quadtree_tile_entity,
    ) != TileVisibility::NONE
    {
        return visitTile(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            quadtree_tile_entity,
        );
    }
    tile_quad_tree
        .replacement_queue
        .markTileRendered(quadtree_tile_query, quadtree_tile_entity);
    let (
        entity,
        globe_surface_tile,
        rectangle,
        parent,
        mut other_state,
        mut replacement_state,
        data,
        key,
        node_id,
        node_children,
        state,
        location,
        _,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let mut entity_mut = commands.entity(entity);

    let traversalDetails = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    traversalDetails.allAreRenderable = true;
    traversalDetails.anyWereRenderedLastFrame = false;
    traversalDetails.notYetRenderableCount = 0;
    if containsNeededPosition(&rectangle, tile_quad_tree) {
        if data.0.is_none() || data.0.as_ref().unwrap().vertexArray.is_none() {
            entity_mut.insert(TileLoadMedium);
        }

        let lastFrame = &tile_quad_tree._lastSelectionFrameNumber;
        let lastFrameSelectionResult = if other_state._lastSelectionResultFrame == *lastFrame {
            &other_state._lastSelectionResult
        } else {
            &TileSelectionResult::NONE
        };
        if (*lastFrameSelectionResult != TileSelectionResult::CULLED_BUT_NEEDED
            && *lastFrameSelectionResult != TileSelectionResult::RENDERED)
        {
            // tile_quad_tree._tileToUpdateHeights.push(tile);
            entity_mut.insert(TileToUpdateHeight);
        }

        other_state._lastSelectionResult = TileSelectionResult::CULLED_BUT_NEEDED;
    } else if (tile_quad_tree.preloadSiblings || key.level == 0) {
        // Load culled level zero tiles with low priority.
        // For all other levels, only load culled tiles if preloadSiblings is enabled.
        entity_mut.insert(TileLoadLow);
        other_state._lastSelectionResult = TileSelectionResult::CULLED;
    } else {
        other_state._lastSelectionResult = TileSelectionResult::CULLED;
    }

    other_state._lastSelectionResultFrame = Some(frame_count.0);
}

// type QueueParmaSet<'world, 'state> = ParamSet<
//     'world,
//     'state,
//     (
//         Query<'world, 'state, Entity, With<TileToRender>>,
//         Query<'world, 'state, Entity, With<TileToUpdateHeight>>,
//         Query<'world, 'state, Entity, With<TileLoadHigh>>,
//         Query<'world, 'state, Entity, With<TileLoadMedium>>,
//         Query<'world, 'state, Entity, With<TileLoadLow>>,
//     ),
// >;
pub type GlobeSurfaceTileQuery<'a> = (
    Entity,
    &'a mut GlobeSurfaceTile,
    &'a Rectangle,
    &'a TileNode,
    &'a mut QuadtreeTileOtherState,
    &'a mut TileReplacementState,
    &'a QuadtreeTileData,
    &'a TileKey,
    &'a TileNode,
    &'a mut NodeChildren,
    &'a QuadtreeTileLoadState,
    &'a Quadrant,
    &'a QuadtreeTileParent,
);
#[derive(Clone, Copy)]
struct TraversalDetails {
    allAreRenderable: bool,
    anyWereRenderedLastFrame: bool,
    notYetRenderableCount: u32,
}
impl Default for TraversalDetails {
    fn default() -> Self {
        Self {
            allAreRenderable: true,
            anyWereRenderedLastFrame: false,
            notYetRenderableCount: 0,
        }
    }
}
#[derive(Resource)]
struct AllTraversalQuadDetails([TraversalQuadDetails; 31]);
impl AllTraversalQuadDetails {
    pub fn new() -> Self {
        AllTraversalQuadDetails([TraversalQuadDetails::new(); 31])
    }
    pub fn get(&self, level: u32) -> &TraversalQuadDetails {
        self.0.get(level as usize).unwrap()
    }
    pub fn get_mut(&mut self, level: u32) -> &mut TraversalQuadDetails {
        self.0.get_mut(level as usize).unwrap()
    }
}
#[derive(Resource)]
struct RootTraversalDetails(Vec<TraversalDetails>);
impl RootTraversalDetails {
    pub fn new() -> Self {
        RootTraversalDetails(Vec::new())
    }
    pub fn get(&self, level: u32) -> &TraversalDetails {
        self.0.get(level as usize).unwrap()
    }
    pub fn get_mut(&mut self, level: u32) -> &mut TraversalDetails {
        self.0.get_mut(level as usize).unwrap()
    }
}
#[derive(Clone, Copy)]
struct TraversalQuadDetails {
    southwest: TraversalDetails,
    southeast: TraversalDetails,
    northwest: TraversalDetails,
    northeast: TraversalDetails,
}
impl TraversalQuadDetails {
    fn new() -> Self {
        Self {
            southwest: TraversalDetails::default(),
            southeast: TraversalDetails::default(),
            northwest: TraversalDetails::default(),
            northeast: TraversalDetails::default(),
        }
    }
    fn combine(&self) -> TraversalDetails {
        let southwest = self.southwest;
        let southeast = self.southeast;
        let northwest = self.northwest;
        let northeast = self.northeast;
        let mut result = TraversalDetails::default();
        result.allAreRenderable = southwest.allAreRenderable
            && southeast.allAreRenderable
            && northwest.allAreRenderable
            && northeast.allAreRenderable;
        result.anyWereRenderedLastFrame = southwest.anyWereRenderedLastFrame
            || southeast.anyWereRenderedLastFrame
            || northwest.anyWereRenderedLastFrame
            || northeast.anyWereRenderedLastFrame;
        result.notYetRenderableCount = southwest.notYetRenderableCount
            + southeast.notYetRenderableCount
            + northwest.notYetRenderableCount
            + northeast.notYetRenderableCount;
        return result;
    }
}
fn screenSpaceError(
    key: &TileKey,
    other_state: &QuadtreeTileOtherState,
    globe_camera: &GlobeCamera,
    window: &Window,
    ellipsoid: &Ellipsoid,
    tiling_scheme: &GeographicTilingScheme,
) -> f64 {
    let maxGeometricError: f64 = getLevelMaximumGeometricError(ellipsoid, tiling_scheme, key.level);

    let distance = other_state._distance;
    let height = window.height() as f64;
    let sseDenominator = globe_camera.frustum.sseDenominator();

    let mut error = (maxGeometricError * height) / (distance * sseDenominator);

    error /= window.scale_factor();

    return error;
}
fn containsNeededPosition(
    rectangle: &Rectangle,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
) -> bool {
    return tile_quad_tree._cameraPositionCartographic.is_some()
        && rectangle.contains(&tile_quad_tree._cameraPositionCartographic.unwrap())
        || tile_quad_tree
            ._cameraReferenceFrameOriginCartographic
            .is_some()
            && rectangle.contains(
                &tile_quad_tree
                    ._cameraReferenceFrameOriginCartographic
                    .unwrap(),
            );
}
fn make_new_quadtree_tile(
    commands: &mut Commands,
    key: TileKey,
    rectangle: Rectangle,
    location: Quadrant,
    parent: QuadtreeTileParent,
) -> TileNode {
    let mut entity_mut = commands.spawn(QuadtreeTile::new(key, rectangle, location, parent));
    let entity = entity_mut.id();
    let node_id = TileNode::Internal(entity_mut.id());
    entity_mut.insert((TileReplacementState::new(entity), node_id.clone()));
    return node_id;
}
fn subdivide(
    commands: &mut Commands,
    node_id: &TileNode,
    key: &TileKey,
    children: &mut NodeChildren,
    tiling_scheme: &GeographicTilingScheme,
) {
    if let TileNode::Internal(index) = node_id {
        let southwest = key.southwest();
        let southwest_rectangle =
            tiling_scheme.tile_x_y_to_rectange(southwest.x, southwest.y, southwest.level);
        let southeast = key.southeast();
        let southeast_rectangle =
            tiling_scheme.tile_x_y_to_rectange(southeast.x, southeast.y, southeast.level);
        let northwest = key.northwest();
        let northwest_rectangle =
            tiling_scheme.tile_x_y_to_rectange(northwest.x, northwest.y, northwest.level);
        let northeast = key.northeast();
        let northeast_rectangle =
            tiling_scheme.tile_x_y_to_rectange(northeast.x, northeast.y, northeast.level);
        let nw = make_new_quadtree_tile(
            commands,
            southwest,
            southwest_rectangle,
            Quadrant::Southwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let ne = make_new_quadtree_tile(
            commands,
            southeast,
            southeast_rectangle,
            Quadrant::Southeast,
            QuadtreeTileParent(node_id.clone()),
        );
        let sw = make_new_quadtree_tile(
            commands,
            northwest,
            northwest_rectangle,
            Quadrant::Northwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let se = make_new_quadtree_tile(
            commands,
            northeast,
            northeast_rectangle,
            Quadrant::Northeast,
            QuadtreeTileParent(node_id.clone()),
        );

        children.northwest = nw;
        children.northeast = ne;
        children.southwest = sw;
        children.southeast = se;
    }
}
fn getEstimatedLevelZeroGeometricErrorForAHeightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    numberOfTilesAtLevelZero: u32,
) -> f64 {
    return ((ellipsoid.maximumRadius * 2. * PI * 0.25)
        / (tile_image_width as f64 * numberOfTilesAtLevelZero as f64));
}
fn get_levelZeroMaximumGeometricError(
    ellipsoid: &Ellipsoid,
    tiling_scheme: &GeographicTilingScheme,
) -> f64 {
    return getEstimatedLevelZeroGeometricErrorForAHeightmap(
        ellipsoid,
        64,
        tiling_scheme.get_number_of_tiles_at_level(0),
    );
}
fn getLevelMaximumGeometricError(
    ellipsoid: &Ellipsoid,
    tiling_scheme: &GeographicTilingScheme,
    level: u32,
) -> f64 {
    let _levelZeroMaximumGeometricError =
        get_levelZeroMaximumGeometricError(ellipsoid, tiling_scheme);
    return _levelZeroMaximumGeometricError / (1 << level) as f64;
}
fn canRenderWithoutLosingDetail() -> bool {
    return true;
}
fn visitVisibleChildrenNearToFar(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    culling_volume: &CullingVolume,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    tiling_scheme: &GeographicTilingScheme,
    ancestorMeetsSse: bool,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
        &World,
    )>,

    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    southwest: &TileNode,
    southeast: &TileNode,
    northwest: &TileNode,
    northeast: &TileNode,
    quadtree_tile_entity: Entity,
) {
    let get_tile_ndoe_entity = |node_id: &TileNode| -> Option<Entity> {
        if let TileNode::Internal(v) = node_id {
            Some(v.clone())
        } else {
            None
        }
    };
    let southwest_entity = get_tile_ndoe_entity(southwest).expect("data不存在");
    let southeast_entity = get_tile_ndoe_entity(southeast).expect("data不存在");
    let northwest_entity = get_tile_ndoe_entity(northwest).expect("data不存在");
    let northeast_entity = get_tile_ndoe_entity(northeast).expect("data不存在");
    let (east, west, south, north, level) = {
        let v = quadtree_tile_query.get(southwest_entity).unwrap();
        (v.2.east, v.2.west, v.2.south, v.2.north, v.7.level)
    };

    let cameraPositionCartographic = globe_camera.get_position_cartographic();
    if (cameraPositionCartographic.longitude < east) {
        if (cameraPositionCartographic.latitude < north) {
            // Camera in southwest quadrant
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southwest_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southeast_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northwest_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northeast_entity,
            );
        } else {
            // Camera in northwest quadrant
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northwest_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southwest_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                northeast_entity,
            );
            visitIfVisible(
                commands,
                tile_quad_tree,
                ellipsoid,
                ellipsoidalOccluder,
                quadtree_tile_query,
                culling_volume,
                frame_count,
                globe_camera,
                window,
                tiling_scheme,
                ancestorMeetsSse,
                all_traversal_quad_details,
                root_traversal_details,
                queue_params_set,
                southeast_entity,
            );
        }
    } else if (cameraPositionCartographic.latitude < north) {
        // Camera southeast quadrant
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southeast_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southwest_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northeast_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northwest_entity,
        );
    } else {
        // Camera in northeast quadrant
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northeast_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            northwest_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southeast_entity,
        );
        visitIfVisible(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            culling_volume,
            frame_count,
            globe_camera,
            window,
            tiling_scheme,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southwest_entity,
        );
    }
    let (_, _, _, _, _, _, _, key, _, _, _, location, _) =
        quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let quadDetailsCombine = { all_traversal_quad_details.get_mut(level).combine() };
    let traversalDetails = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    *traversalDetails = quadDetailsCombine;
}
fn remove_component<T: Component>(
    commands: &mut Commands,
    queue_query: &Query<(Entity), With<T>>,
    length: usize,
) {
    queue_query.iter().enumerate().for_each(|(i, x)| {
        if i > length - 1 {
            commands.entity(x).remove::<T>();
        }
    })
}
fn end_frame(
    mut commands: Commands,
    mut queue_params_set: ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
    )>,
) {
    for i in queue_params_set.p0().iter() {}
}
