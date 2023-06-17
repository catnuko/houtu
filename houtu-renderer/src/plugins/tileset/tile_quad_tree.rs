use std::cmp::Ordering;
use std::sync::Arc;

use bevy::core::FrameCount;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use houtu_scene::{
    Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Matrix4,
    Rectangle, TileBoundingRegion, TilingScheme,
};

use crate::plugins::camera::GlobeCamera;

use super::globe_surface_tile::{computeTileVisibility, GlobeSurfaceTile, TileVisibility};
use super::tile_selection_result::TileSelectionResult;
use super::tile_tree::TileTree;
use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileData, QuadtreeTileLoadState,
    QuadtreeTileMark, QuadtreeTileOtherState, TileLoadHigh, TileLoadLow, TileLoadMedium, TileNode,
    TileToRender, TileToUpdateHeight,
};
use super::tile_datasource::{self, QuadTreeTileDatasourceMark, Ready, TilingSchemeWrap};
use super::tile_replacement_queue::{self, TileReplacementQueue, TileReplacementState};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(tile_datasource::Plugin);
        app.add_system(
            begin_frame
                .before(render::<GeographicTilingScheme>)
                .before(end_frame),
        );
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
    tiletree: TileTree,
    replacement_queue: TileReplacementQueue,
}

impl TileQuadTree {
    pub fn new(tiling_scheme: Arc<GeographicTilingScheme>) -> Self {
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
            tiletree: TileTree::new(tiling_scheme),
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
fn render<T: TilingScheme + Sync + Send + 'static>(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    render_queue_query: Query<Entity, With<TileToRender>>,
    mut datasource_query: Query<(&Ready, &TilingSchemeWrap<T>), With<QuadTreeTileDatasourceMark>>,
    mut tile_quad_node_query: Query<
        (
            Entity,
            &Quadrant,
            &Rectangle,
            &TileReplacementState,
            &QuadtreeTileOtherState,
        ),
        With<QuadtreeTileMark>,
    >,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
) {
    //selectTilesForRendering
    //清空渲染列表
    render_queue_query.iter().for_each(|entity: Entity| {
        let mut entity_mut = commands.get_entity(entity).expect("entity不存在");
        entity_mut.remove::<TileToRender>();
    });
    if datasource_query.iter().len() != 1 {
        return;
    }
    let (globe_camera) = globe_camera_query.get_single().expect("GlobeCamera不存在");
    let datasource = datasource_query
        .get_single()
        .expect("QuadTreeTileDatasourceMark不存在");
    //创建根节点
    if tile_quad_node_query
        .iter()
        .filter(|(_, location, _, _, _)| **location == Quadrant::Root)
        .count()
        == 0
    {
        let (ready, tiling_scheme_wrap) = datasource;
        if ready.0 {
            let tiling_scheme = tiling_scheme_wrap.0;
            let numberOfLevelZeroTilesX = tiling_scheme.get_number_of_x_tiles_at_level(0);
            let numberOfLevelZeroTilesY = tiling_scheme.get_number_of_y_tiles_at_level(0);
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
                        Quadrant::Root,
                        TileNode::None,
                    );
                }
            }
        } else {
            return;
        }
    }
    let occluders = if tile_quad_node_query.iter().count() > 1 {
        Some(EllipsoidalOccluder::default())
    } else {
        None
    };
    //按相机位置排序，从近到远
    let p = globe_camera.get_position_cartographic();
    let mut tt = vec![];
    tile_quad_node_query.iter_mut().for_each(|x| tt.push(x));
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
    for (entity, location, rectangle, mut replacement_state, other_state) in tt {
        let mut entity_mut = commands.entity(entity);
        tile_quad_tree
            .replacement_queue
            .markTileRendered(&mut commands, &mut replacement_state);
        if !other_state.renderable {
            entity_mut.insert(TileLoadHigh);
        } else {
            visitIfVisible()
        }
    }
}

fn visitTile(
    commands: &mut Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut (
        Entity,
        &GlobeSurfaceTile,
        &Rectangle,
        &TileNode,
        &QuadtreeTileOtherState,
        &mut TileReplacementState,
        &QuadtreeTileData,
        &TileKey,
        &TileNode,
        &mut NodeChildren,
        &QuadtreeTileLoadState,
    ),
    camera: &mut GlobeCamera,
    culling_volume: &CullingVolume,
    traversalDetails: &mut TraversalDetails,
    mut frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    tiling_scheme: &GeographicTilingScheme,

    ancestorMeetsSse: bool,
) {
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
    ) = quadtree_tile_query;
    let entity_mut = commands.entity(entity);
    tile_quad_tree
        .replacement_queue
        .markTileRendered(commands, replacement_state);
    let meetsSse = screenSpaceError(key, other_state, globe_camera, window)
        < tile_quad_tree.maximumScreenSpaceError;
    subdivide(commands, node_id, key, node_children, tiling_scheme);
    let southwestChild = node_children.southwest;
    let southeastChild = node_children.southeast;
    let northwestChild = node_children.northwest;
    let northeastChild = node_children.northeast;

    let lastFrame = tile_quad_tree._lastSelectionFrameNumber;
    let lastFrameSelectionResult = if other_state._lastSelectionResultFrame == lastFrame {
        other_state._lastSelectionResult
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

        let oneRenderedLastFrame = TileSelectionResult::originalResult(lastFrameSelectionResult)
            == TileSelectionResult::RENDERED;
        let twoCulledOrNotVisited = TileSelectionResult::originalResult(lastFrameSelectionResult)
            == TileSelectionResult::CULLED
            || lastFrameSelectionResult == TileSelectionResult::NONE;
        let threeCompletelyLoaded = state == QuadtreeTileLoadState::DONE;

        let renderable = oneRenderedLastFrame || twoCulledOrNotVisited || threeCompletelyLoaded;

        if (!renderable) {
            // Check the more expensive condition 4 above. This requires details of the thing
            // we're rendering (e.g. the globe surface), so delegate it to the tile provider.
            if (defined(tileProvider.canRenderWithoutLosingDetail)) {
                renderable = tileProvider.canRenderWithoutLosingDetail(tile);
            }
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
}
fn visitIfVisible(
    commands: &mut Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut (
        Entity,
        &GlobeSurfaceTile,
        &Rectangle,
        &TileNode,
        &QuadtreeTileOtherState,
        &mut TileReplacementState,
        &QuadtreeTileData,
        &TileKey,
        &TileNode,
        &mut NodeChildren,
        &QuadtreeTileLoadState,
    ),
    camera: &mut GlobeCamera,
    culling_volume: &CullingVolume,
    traversalDetails: &mut TraversalDetails,
    mut frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    tiling_scheme: &GeographicTilingScheme,
) {
    if computeTileVisibility(
        commands,
        ellipsoid,
        ellipsoidalOccluder,
        (
            quadtree_tile_query.0,
            quadtree_tile_query.1,
            quadtree_tile_query.2,
            quadtree_tile_query.3,
            quadtree_tile_query.4,
        ),
        camera,
        culling_volume,
    ) != TileVisibility::NONE
    {
        return visitTile();
    }
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
    ) = quadtree_tile_query;
    let mut entity_mut = commands.entity(*entity);
    tile_quad_tree
        .replacement_queue
        .markTileRendered(commands, replacement_state);
    traversalDetails.allAreRenderable = true;
    traversalDetails.anyWereRenderedLastFrame = false;
    traversalDetails.notYetRenderableCount = 0;
    if containsNeededPosition(&rectangle, tile_quad_tree) {
        if data.0.is_none() || data.0.unwrap().vertexArray.is_none() {
            entity_mut.insert(TileLoadMedium);
        }

        let lastFrame = tile_quad_tree._lastSelectionFrameNumber;
        let lastFrameSelectionResult = if other_state._lastSelectionResultFrame == lastFrame {
            other_state._lastSelectionResult
        } else {
            TileSelectionResult::NONE
        };
        if (lastFrameSelectionResult != TileSelectionResult::CULLED_BUT_NEEDED
            && lastFrameSelectionResult != TileSelectionResult::RENDERED)
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

fn end_frame(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    high_queue_query: Query<Entity, With<TileLoadHigh>>,
    medium_queue_query: Query<Entity, With<TileLoadMedium>>,
    low_queue_query: Query<Entity, With<TileLoadLow>>,
    render_queue_query: Query<Entity, With<TileToRender>>,
) {
}

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
struct TraversalQuadDetails {
    southwest: TraversalDetails,
    southeast: TraversalDetails,
    northwest: TraversalDetails,
    northeast: TraversalDetails,
}
impl TraversalQuadDetails {
    fn combin(&self) -> TraversalDetails {
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
) -> f64 {
    let maxGeometricError: f64 = datasource.getLevelMaximumGeometricError(key.level);

    let distance = other_state._distance;
    let height = window.height() as f64;
    let sseDenominator = globe_camera.frustum.sseDenominator();

    let error = (maxGeometricError * height) / (distance * sseDenominator);

    error /= window.scale_factor();

    return error;
}
fn containsNeededPosition(rectangle: &Rectangle, mut tile_quad_tree: ResMut<TileQuadTree>) -> bool {
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
fn make_new_quadtree_tile<'a>(
    commands: &'a mut Commands<'a, 'a>,
    key: TileKey,
    rectangle: Rectangle,
    location: Quadrant,
    parent: TileNode,
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
            node_id.clone(),
        );
        let ne = make_new_quadtree_tile(
            commands,
            southeast,
            southeast_rectangle,
            Quadrant::Southeast,
            node_id.clone(),
        );
        let sw = make_new_quadtree_tile(
            commands,
            northwest,
            northwest_rectangle,
            Quadrant::Northwest,
            node_id.clone(),
        );
        let se = make_new_quadtree_tile(
            commands,
            northeast,
            northeast_rectangle,
            Quadrant::Northeast,
            node_id.clone(),
        );

        children.northwest = nw;
        children.northeast = ne;
        children.southwest = sw;
        children.southeast = se;
    }
}
