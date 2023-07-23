use std::cmp::Ordering;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use bevy::core::FrameCount;
use bevy::ecs::system::{EntityCommands, QueryComponentError};
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy::window::PrimaryWindow;
use houtu_jobs::{FinishedJobs, JobSpawner};
use houtu_scene::{
    Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme,
    HeightmapTerrainData, IndicesAndEdgesCache, Matrix4, Rectangle, TerrainExaggeration,
    TerrainMesh, TileBoundingRegion, TilingScheme,
};
use rand::Rng;

use crate::plugins::camera::GlobeCamera;

use super::create_terrain_mesh_job::CreateTileJob;
use super::globe_surface_tile::{
    self, computeTileVisibility, GlobeSurfaceTile, TerrainState, TileVisibility,
};
use super::imagery::{Imagery, ImageryState};
use super::imagery_layer::{
    self, ImageryLayer, ImageryLayerOtherState, TerrainDataSource, XYZDataSource,
};
use super::reproject_texture::{self, ReprojectTextureTaskQueue};
use super::terrian_material::TerrainMeshMaterial;
use super::tile_selection_result::TileSelectionResult;
use super::traversal_details::{AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails};
use super::upsample_job::UpsampleJob;
use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileData, QuadtreeTileLoadState,
    QuadtreeTileMark, QuadtreeTileOtherState, QuadtreeTileParent, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileToLoad, TileToRender, TileToUpdateHeight,
};
use super::tile_replacement_queue::{TileReplacementQueue, TileReplacementState};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(imagery_layer::Plugin);
        app.register_type::<TileKey>()
            .register_type::<TileReplacementState>()
            .register_type::<Quadrant>()
            .register_type::<NodeChildren>()
            .register_type::<QuadtreeTileMark>()
            .register_type::<QuadtreeTileParent>()
            .register_type::<TileToRender>()
            .register_type::<TileToUpdateHeight>()
            .register_type::<TileLoadHigh>()
            .register_type::<TileLoadMedium>()
            .register_type::<TileLoadLow>()
            .register_type::<TileToLoad>();

        app.insert_resource(TileQuadTree::new());
        app.insert_resource(AllTraversalQuadDetails::new());
        app.insert_resource(RootTraversalDetails::new());
        app.insert_resource(IndicesAndEdgesCacheArc::new());
        app.add_event::<TileLoadEvent>();
        app.add_system(begin_frame.before(render).before(end_frame));
        app.add_system(render.before(end_frame));
        app.add_system(end_frame.before(updateTileLoadProgress_system));
        app.add_system(updateTileLoadProgress_system);
        app.add_systems((
            quad_tile_state_init_system,
            terrain_state_machine_system,
            upsample_system,
            request_tile_geometry_system,
            transform_system,
            quad_tile_state_end_system,
        ));
        app.add_system(ImageryLayer::finish_reproject_texture_system);
        app.add_system(quadtree_tile_load_state_done_system);
    }
}
#[derive(Resource)]
pub struct IndicesAndEdgesCacheArc(pub Arc<Mutex<IndicesAndEdgesCache>>);
impl IndicesAndEdgesCacheArc {
    fn new() -> Self {
        IndicesAndEdgesCacheArc(Arc::new(Mutex::new(IndicesAndEdgesCache::new())))
    }
    fn get_cloned_cache(&self) -> Arc<Mutex<IndicesAndEdgesCache>> {
        return self.0.clone();
    }
}
#[derive(Resource)]
pub struct TileQuadTree {
    tileCacheSize: u32,
    maximumScreenSpaceError: f64,
    _loadQueueTimeSlice: u32,
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
    debug: TileQuadTreeDebug,
}

impl TileQuadTree {
    pub fn new() -> Self {
        Self {
            tileCacheSize: 100,
            loadingDescendantLimit: 20,
            preloadAncestors: true,
            _loadQueueTimeSlice: 5,
            _tilesInvalidated: false,
            maximumScreenSpaceError: 2.0,
            preloadSiblings: false,
            _lastTileLoadQueueLength: 0,
            _lastSelectionFrameNumber: None,
            _occluders: EllipsoidalOccluder::default(),
            _cameraPositionCartographic: None,
            _cameraReferenceFrameOriginCartographic: None,
            replacement_queue: TileReplacementQueue::new(),
            debug: TileQuadTreeDebug::new(),
        }
    }
    /// 调用后将清空所有瓦片重新创建
    pub fn invalidateAllTiles(&mut self) {
        self._tilesInvalidated = true;
    }
    pub fn real_invalidateAllTiles(&mut self) {}
}
pub struct TileQuadTreeDebug {
    enableDebugOutput: bool,

    maxDepth: u32,
    maxDepthVisited: u32,
    tilesVisited: u32,
    tilesCulled: u32,
    tilesRendered: u32,
    tilesWaitingForChildren: u32,

    lastMaxDepth: u32,
    lastMaxDepthVisited: u32,
    lastTilesVisited: u32,
    lastTilesCulled: u32,
    lastTilesRendered: u32,
    lastTilesWaitingForChildren: u32,

    suspendLodUpdate: bool,
}
impl TileQuadTreeDebug {
    pub fn new() -> Self {
        Self {
            enableDebugOutput: true,
            maxDepth: 0,
            maxDepthVisited: 0,
            tilesVisited: 0,
            tilesCulled: 0,
            tilesRendered: 0,
            tilesWaitingForChildren: 0,
            lastMaxDepth: 0,
            lastMaxDepthVisited: 0,
            lastTilesVisited: 0,
            lastTilesCulled: 0,
            lastTilesRendered: 0,
            lastTilesWaitingForChildren: 0,
            suspendLodUpdate: false,
        }
    }
    pub fn reset(&mut self) {
        self.maxDepth = 0;
        self.maxDepthVisited = 0;
        self.tilesVisited = 0;
        self.tilesCulled = 0;
        self.tilesRendered = 0;
        self.tilesWaitingForChildren = 0;
    }
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
    tile_quad_tree.debug.reset();
    if tile_quad_tree.debug.suspendLodUpdate {
        return;
    }
    tile_quad_tree.replacement_queue.markStartOfRenderFrame();
    // TODO createRenderCommandsForSelectedTiles函数开始
}
fn render(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    render_queue_query: Query<Entity, With<TileToRender>>,
    mut datasource_query: Query<&mut TerrainDataSource>,
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery>,
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
    )>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    //selectTilesForRendering
    if tile_quad_tree.debug.suspendLodUpdate {
        return;
    }
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
    let mut terrain_datasource = datasource_query
        .get_single_mut()
        .expect("QuadTreeTileDatasourceMark不存在");
    //创建根节点
    let root_count = quadtree_tile_query
        .iter()
        .filter(|v| {
            if let Quadrant::Root(_) = *v.9 {
                return true;
            } else {
                return false;
            }
        })
        .count();
    if root_count == 0 {
        // let (ready, tiling_scheme_wrap) = terrain_datasource;
        if terrain_datasource.ready {
            let numberOfLevelZeroTilesX = terrain_datasource
                .tiling_scheme
                .get_number_of_x_tiles_at_level(0);
            let numberOfLevelZeroTilesY = terrain_datasource
                .tiling_scheme
                .get_number_of_y_tiles_at_level(0);
            let mut i = 0;
            for y in 0..numberOfLevelZeroTilesY {
                for x in 0..numberOfLevelZeroTilesX {
                    let r = terrain_datasource
                        .tiling_scheme
                        .tile_x_y_to_rectange(x, y, 0);

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
            if root_traversal_details.0.len() < i {
                root_traversal_details.0 = vec![TraversalDetails::default(); i];
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
    quadtree_tile_query
        .iter()
        .for_each(|x| tt.push((x.0.clone(), x.2.clone())));
    tt.sort_by(|a, b| {
        let mut center = a.1.center();
        let alon = center.longitude - p.longitude;
        let alat = center.latitude - p.latitude;
        center = b.1.center();
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
        let (entity, _) = x;
        tile_quad_tree
            .replacement_queue
            .markTileRendered(&mut quadtree_tile_query, *entity);
        let mut other_state = quadtree_tile_query
            .get_component_mut::<QuadtreeTileOtherState>(*entity)
            .unwrap();
        if !other_state.renderable {
            commands.entity(*entity).insert(TileLoadHigh);
            tile_quad_tree.debug.tilesWaitingForChildren += 1;
        } else {
            visitIfVisible(
                &mut commands,
                &mut tile_quad_tree,
                &ellipsoidalOccluder.ellipsoid,
                &ellipsoidalOccluder,
                &mut quadtree_tile_query,
                &frame_count,
                &mut globe_camera,
                window,
                terrain_datasource.as_mut(),
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
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestorMeetsSse: bool,
    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
    )>,
    quadtree_tile_entity: Entity,
) {
    tile_quad_tree.debug.tilesVisited += 1;

    let mut ancestorMeetsSse = ancestorMeetsSse;
    tile_quad_tree
        .replacement_queue
        .markTileRendered(quadtree_tile_query, quadtree_tile_entity);
    let (
        entity,
        globe_surface_tile,
        rectangle,
        mut other_state,
        mut replacement_state,
        key,
        node_id,
        mut node_children,
        state,
        location,
        parent,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    // info!("visitTile key={:?}", key);
    if key.level > tile_quad_tree.debug.maxDepthVisited {
        tile_quad_tree.debug.maxDepthVisited = key.level;
    }
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
        terrain_datasource,
    ) < tile_quad_tree.maximumScreenSpaceError;
    subdivide(
        entity_mut.commands(),
        node_id,
        key,
        &mut node_children,
        terrain_datasource,
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

        /*
        当前瓦片或者祖先瓦片时我们想要在这帧渲染的，但是，根据当前瓦片的状态和我们上帧做的事情的不同，将做一些不同的事情
        我们将渲染当前瓦片如果下列任一条件满足：
        1. 我们在上帧渲染过或者踢出过
        2. 这个瓦片在上帧被视锥体裁剪了或者由于它的祖先瓦片被裁剪导致它不可见
        3. 当前瓦片完全加载完成了
        4. a) 地形已经准备好了
           b) 所有必要的图片准备好了，必要的图片是指当前瓦片和当前瓦片的子孙瓦片需要的图片。之所以需要，是因为没有它们将丢失细节。
        */
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
            /*
            上面四个条件满足其一时可渲染
            */
            renderable = false
        }

        if (renderable) {
            // Only load this tile if it (not just an ancestor) meets the SSE.
            // 只有当前瓦片满足SSE时再去加载所需资源
            if (meetsSse) {
                entity_mut.insert(TileLoadMedium);
            }
            entity_mut.insert(TileToRender);
            entity_mut.remove::<(TileToLoad)>();

            traversalDetails.all_are_renderable = other_state.renderable;
            traversalDetails.any_were_rendered_last_frame =
                lastFrameSelectionResult == TileSelectionResult::RENDERED;
            traversalDetails.not_yet_renderable_count = if other_state.renderable { 0 } else { 1 };

            other_state._lastSelectionResultFrame = Some(frame_count.0);
            other_state._lastSelectionResult = TileSelectionResult::RENDERED;

            if (!traversalDetails.any_were_rendered_last_frame) {
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
        /*
        否则，我们不能渲染当前瓦片，这样做会造成上一帧就有的细节丢失。相反，继续渲染上一帧渲染过的可见的子孙瓦片，刚刚可见的子孙瓦片。
        如果我们上一帧在渲染15层，但是这一帧渲染14层和最近的可渲染层<=14是0。渲染0层肯定是不合适的，所以我们继续渲染15层，即使他的SSE比需要的更好。
        所以要继续遍历。
         */
        ancestorMeetsSse = true;

        // Load this blocker tile with high priority, but only if this tile (not just an ancestor) meets the SSE.
        if (meetsSse) {
            entity_mut.insert(TileLoadHigh);
        }
    }

    if terrain_datasource.canRefine(&globe_surface_tile, key) {
        let mut allAreUpsampled = true;
        if let TileNode::Internal(v) = southwestChild {
            if let Ok(state) = quadtree_tile_query.get_component::<QuadtreeTileOtherState>(v) {
                allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
            } else {
                return;
            }
        }
        if let TileNode::Internal(v) = southeastChild {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }
        if let TileNode::Internal(v) = northwestChild {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }
        if let TileNode::Internal(v) = northeastChild {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            allAreUpsampled = allAreUpsampled && state.upsampledFromParent;
        }

        if (allAreUpsampled) {
            // No point in rendering the children because they're all upsampled.  Render this tile instead.
            // 没必要渲染子孙瓦片，因为他们都是被采样过的。渲染当前瓦片。
            entity_mut.insert(TileToRender);

            // Rendered tile that's not waiting on children loads with medium priority.
            // 渲染瓦片，不等待中等优先级的子孙瓦片加载
            entity_mut.insert(TileLoadHigh);

            // Make sure we don't unload the children and forget they're upsampled.
            // 确保我们没卸载子孙瓦片忘记他们被采样过。
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
            traversalDetails.all_are_renderable = other_state.renderable;
            traversalDetails.any_were_rendered_last_frame =
                lastFrameSelectionResult == TileSelectionResult::RENDERED;
            traversalDetails.not_yet_renderable_count = if other_state.renderable { 0 } else { 1 };

            other_state._lastSelectionResultFrame = Some(frame_count.0);
            other_state._lastSelectionResult = TileSelectionResult::RENDERED;

            if (!traversalDetails.any_were_rendered_last_frame) {
                // Tile is newly-rendered this frame, so update its heights.
                entity_mut.insert(TileToUpdateHeight);
            }

            return;
        }
        // SSE is not good enough, so refine.
        // SSE不太好，所以细分
        let mut other_state = quadtree_tile_query
            .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
            .unwrap();
        other_state._lastSelectionResultFrame = Some(frame_count.0);
        other_state._lastSelectionResult = TileSelectionResult::REFINED;

        let firstRenderedDescendantIndex = queue_params_set.p0().iter().count();
        let loadIndexLow = queue_params_set.p4().iter().count();
        let loadIndexMedium = queue_params_set.p3().iter().count();
        let loadIndexHigh = queue_params_set.p2().iter().count();
        let tilesToUpdateHeightsIndex = queue_params_set.p1().iter().count();

        // No need to add the children to the load queue because they'll be added (if necessary) when they're visited.
        // 不需要将子孙放入加载队列，因为他们被visited时会被加载
        visitVisibleChildrenNearToFar(
            entity_mut.commands(),
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
        /*
        如果上面的函数没有添加子孙瓦片到渲染列表中，意味着他们子孙瓦片们都被剔除了，即使当前瓦片时可见的。这种情况很常见。
        如果最初的渲染列表长度不等于当前的渲染列表长度，则添加了子孙瓦片到渲染列表中
         */

        if (firstRenderedDescendantIndex != render_count) {
            // At least one descendant tile was added to the render list.
            // The traversalDetails tell us what happened while visiting the children.
            /*
            至少一个子孙瓦片被添加到渲染列表中。traversalDetails告诉我们遍历子孙时发生了什么。
             */

            let all_are_renderable = traversalDetails.all_are_renderable;
            let any_were_rendered_last_frame = traversalDetails.any_were_rendered_last_frame;
            let not_yet_renderable_count = traversalDetails.not_yet_renderable_count;
            let mut queuedForLoad = false;
            /*
            如果all_are_renderable==false，意味着当前瓦片和子孙瓦片中有一个瓦片不可被渲染
            如果all_are_renderable==True，意味着当前瓦片和子孙瓦片都可渲染
            如果any_were_rendered_last_frame==false，意味着当前瓦片和子孙瓦片的上帧渲染结果都不等于Rendered
            如果any_were_rendered_last_frame==True，意味着当前瓦片和子孙瓦片的上帧渲染结果中有一个等于Rendered
            如果它俩都等于false，则意味着当前瓦片和子孙瓦片上帧都没被渲染过，而且有一个子孙瓦片不可被渲染
            此时，执行下面的操作，将所有子孙瓦片从渲染列表中踢出，只渲染当前瓦片，继续加载子孙瓦片。
            */
            if (!all_are_renderable && !any_were_rendered_last_frame) {
                // Some of our descendants aren't ready to render yet, and none were rendered last frame,
                // so kick them all out of the render list and render this tile instead. Continue to load them though!

                // Mark the rendered descendants and their ancestors - up to this tile - as kicked.
                /*
                子孙瓦片中的一些还没准备好渲染而且没有一个在上一帧渲染了，所以将所有子孙瓦片从渲染列表中踢出去
                只渲染当前瓦片，继续加载子孙瓦片。

                标记被渲染的子孙瓦片和他们的祖先瓦片(直到当前瓦片),然后踢出去
                 */
                queue_params_set.p0().iter().enumerate().for_each(|(i, e)| {
                    if i >= firstRenderedDescendantIndex {
                        let mut workTile = e.clone();
                        while (workTile != entity) {
                            let mut work_tile = quadtree_tile_query.get_mut(workTile).unwrap();
                            let other_state = &mut work_tile.3;
                            other_state._lastSelectionResult = TileSelectionResult::from_u8(
                                TileSelectionResult::kick(&other_state._lastSelectionResult),
                            );
                            let parent = &work_tile.10;
                            if let QuadtreeTileParent(TileNode::Internal(v)) = parent {
                                workTile = v.clone();
                            }
                        }
                    }
                });

                // Remove all descendants from the render list and add this tile.
                // 移除所有的子孙瓦片和当前瓦片
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
                /*
                如果我们要等一大堆后代，上面的计算将花费很长时间，所以这种情况，加载当前瓦片而不是任何子孙瓦片，并告诉上层？
                我们正在等待当前瓦片加载，继续这样做直到我们能渲染当前瓦片
                 */
                let wasRenderedLastFrame =
                    lastFrameSelectionResult == TileSelectionResult::RENDERED;
                if (!wasRenderedLastFrame
                    && not_yet_renderable_count > tile_quad_tree.loadingDescendantLimit)
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
                    traversalDetails.not_yet_renderable_count =
                        if other_state.renderable { 0 } else { 1 };
                    queuedForLoad = true;
                }

                traversalDetails.all_are_renderable = other_state.renderable;
                traversalDetails.any_were_rendered_last_frame = wasRenderedLastFrame;

                if (!wasRenderedLastFrame) {
                    // Tile is newly-rendered this frame, so update its heights.
                    // 瓦片时这帧刚渲染的，所以更新它的高度
                    entity_mut.insert(TileToUpdateHeight);
                }
                tile_quad_tree.debug.tilesWaitingForChildren += 1;
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
    /*
    我们想要细分，但是，因为我们没有子孙瓦片的可用数据，所以我们不知道细分是否涉及到加载和采样上，
    为了解决，我们不得不先等待当前瓦片加载完成，所以用高优先级加载细分块。
     */
    entity_mut.insert(TileToRender);
    entity_mut.insert(TileLoadHigh);
    traversalDetails.all_are_renderable = renderable;
    traversalDetails.any_were_rendered_last_frame =
        lastFrameSelectionResult == TileSelectionResult::RENDERED;
    traversalDetails.not_yet_renderable_count = if renderable { 0 } else { 1 };
}
fn visitIfVisible(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestorMeetsSse: bool,
    all_traversal_quad_details: &mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &mut ResMut<RootTraversalDetails>,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
    )>,
    quadtree_tile_entity: Entity,
) {
    // info!("visitIfVisible entity={:?}", quadtree_tile_entity);
    if computeTileVisibility(
        // commands,
        // ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
        globe_camera,
        quadtree_tile_entity,
    ) != TileVisibility::NONE
    {
        return visitTile(
            commands,
            tile_quad_tree,
            ellipsoid,
            ellipsoidalOccluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            quadtree_tile_entity,
        );
    }
    tile_quad_tree.debug.tilesCulled += 1;
    tile_quad_tree
        .replacement_queue
        .markTileRendered(quadtree_tile_query, quadtree_tile_entity);
    let (
        entity,
        globe_surface_tile,
        rectangle,
        mut other_state,
        mut replacement_state,
        key,
        node_id,
        node_children,
        state,
        location,
        parent,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let mut entity_mut = commands.entity(entity);

    let traversalDetails = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    traversalDetails.all_are_renderable = true;
    traversalDetails.any_were_rendered_last_frame = false;
    traversalDetails.not_yet_renderable_count = 0;
    if containsNeededPosition(&rectangle, tile_quad_tree) {
        // if data.0.is_none() || data.0.as_ref().unwrap().vertexArray.is_none() {
        //     entity_mut.insert(TileLoadMedium);
        // }

        // let lastFrame = &tile_quad_tree._lastSelectionFrameNumber;
        // let lastFrameSelectionResult = if other_state._lastSelectionResultFrame == *lastFrame {
        //     &other_state._lastSelectionResult
        // } else {
        //     &TileSelectionResult::NONE
        // };
        // if (*lastFrameSelectionResult != TileSelectionResult::CULLED_BUT_NEEDED
        //     && *lastFrameSelectionResult != TileSelectionResult::RENDERED)
        // {
        //     // tile_quad_tree._tileToUpdateHeights.push(tile);
        //     entity_mut.insert(TileToUpdateHeight);
        // }

        // other_state._lastSelectionResult = TileSelectionResult::CULLED_BUT_NEEDED;
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

pub type GlobeSurfaceTileQuery<'a> = (
    Entity,
    &'a mut GlobeSurfaceTile,
    &'a Rectangle,
    &'a mut QuadtreeTileOtherState,
    &'a mut TileReplacementState,
    &'a TileKey,
    &'a TileNode,
    &'a mut NodeChildren,
    &'a mut QuadtreeTileLoadState,
    &'a Quadrant,
    &'a QuadtreeTileParent,
);

fn screenSpaceError(
    key: &TileKey,
    other_state: &QuadtreeTileOtherState,
    globe_camera: &GlobeCamera,
    window: &Window,
    ellipsoid: &Ellipsoid,
    terrain_datasource: &mut TerrainDataSource,
) -> f64 {
    let maxGeometricError: f64 = terrain_datasource.getLevelMaximumGeometricError(key.level);

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
    let node_id = TileNode::Internal(entity);
    entity_mut.insert((TileReplacementState::new(entity), node_id.clone()));
    return node_id;
}
fn subdivide(
    commands: &mut Commands,
    node_id: &TileNode,
    key: &TileKey,
    children: &mut NodeChildren,
    terrain_datasource: &mut TerrainDataSource,
) {
    if let TileNode::Internal(v) = children.southeast {
        return;
    }
    if let TileNode::Internal(index) = node_id {
        let southwest = key.southwest();
        let southwest_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            southwest.x,
            southwest.y,
            southwest.level,
        );
        let southeast = key.southeast();
        let southeast_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            southeast.x,
            southeast.y,
            southeast.level,
        );
        let northwest = key.northwest();
        let northwest_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            northwest.x,
            northwest.y,
            northwest.level,
        );
        let northeast = key.northeast();
        let northeast_rectangle = terrain_datasource.tiling_scheme.tile_x_y_to_rectange(
            northeast.x,
            northeast.y,
            northeast.level,
        );
        let sw = make_new_quadtree_tile(
            commands,
            southwest,
            southwest_rectangle,
            Quadrant::Southwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let se = make_new_quadtree_tile(
            commands,
            southeast,
            southeast_rectangle,
            Quadrant::Southeast,
            QuadtreeTileParent(node_id.clone()),
        );
        let nw = make_new_quadtree_tile(
            commands,
            northwest,
            northwest_rectangle,
            Quadrant::Northwest,
            QuadtreeTileParent(node_id.clone()),
        );
        let ne = make_new_quadtree_tile(
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

fn canRenderWithoutLosingDetail() -> bool {
    return true;
}
fn visitVisibleChildrenNearToFar(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestorMeetsSse: bool,
    queue_params_set: &mut ParamSet<(
        Query<Entity, With<TileToRender>>,
        Query<Entity, With<TileToUpdateHeight>>,
        Query<Entity, With<TileLoadHigh>>,
        Query<Entity, With<TileLoadMedium>>,
        Query<Entity, With<TileLoadLow>>,
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
    info!(
        "visitVisibleChildrenNearToFar key={:?}",
        quadtree_tile_entity
    );
    let southwest_entity = get_tile_ndoe_entity(southwest).expect("data不存在");
    let southeast_entity = get_tile_ndoe_entity(southeast).expect("data不存在");
    let northwest_entity = get_tile_ndoe_entity(northwest).expect("data不存在");
    let northeast_entity = get_tile_ndoe_entity(northeast).expect("data不存在");
    let (east, west, south, north, level) = {
        let v = quadtree_tile_query.get(southwest_entity).unwrap();
        (v.2.east, v.2.west, v.2.south, v.2.north, v.5.level)
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
                frame_count,
                globe_camera,
                window,
                terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
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
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestorMeetsSse,
            all_traversal_quad_details,
            root_traversal_details,
            queue_params_set,
            southwest_entity,
        );
    }
    let (_, _, _, _, _, key, _, _, _, location, _) =
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
    queue_tile_load_high: Query<Entity, With<TileLoadHigh>>,
    queue_tile_load_medium: Query<Entity, With<TileLoadMedium>>,
    queue_tile_load_low: Query<Entity, With<TileLoadLow>>,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    frame_count: Res<FrameCount>,
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery>,
) {
    processTileLoadQueue(
        &queue_tile_load_high,
        &queue_tile_load_medium,
        &queue_tile_load_low,
        &mut tile_quad_tree,
        &mut quadtree_tile_query,
        &frame_count,
        &mut commands,
    );
    //TODO update_heights_system
}

fn processTileLoadQueue(
    queue_tile_load_high: &Query<Entity, With<TileLoadHigh>>,
    queue_tile_load_medium: &Query<Entity, With<TileLoadMedium>>,
    queue_tile_load_low: &Query<Entity, With<TileLoadLow>>,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    commands: &mut Commands,
) {
    if (queue_tile_load_high.iter().count() == 0
        && queue_tile_load_medium.iter().count() == 0
        && queue_tile_load_low.iter().count() == 0)
    {
        return;
    }

    // Remove any tiles that were not used this frame beyond the number
    // we're allowed to keep.
    let size = tile_quad_tree.tileCacheSize;
    tile_quad_tree
        .replacement_queue
        .trimTiles(size, quadtree_tile_query);

    let endTime = frame_count.0 + tile_quad_tree._loadQueueTimeSlice;

    let mut didSomeLoading = false;
    processSinglePriorityLoadQueue(
        frame_count,
        commands,
        endTime,
        &queue_tile_load_high,
        &mut didSomeLoading,
        quadtree_tile_query,
        tile_quad_tree,
    );
    processSinglePriorityLoadQueue(
        frame_count,
        commands,
        endTime,
        &queue_tile_load_medium,
        &mut didSomeLoading,
        quadtree_tile_query,
        tile_quad_tree,
    );
    processSinglePriorityLoadQueue(
        frame_count,
        commands,
        endTime,
        &queue_tile_load_low,
        &mut didSomeLoading,
        quadtree_tile_query,
        tile_quad_tree,
    );
}

fn processSinglePriorityLoadQueue<T: Component>(
    frame_count: &Res<FrameCount>,
    commands: &mut Commands,
    endTime: u32,
    loadQueue: &Query<Entity, With<T>>,
    didSomeLoading: &mut bool,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
) {
    for i in loadQueue.iter() {
        tile_quad_tree
            .replacement_queue
            .markTileRendered(quadtree_tile_query, i);
        commands.entity(i).insert(TileToLoad);
        *didSomeLoading = true;
        if !(frame_count.0 < endTime || !*didSomeLoading) {
            break;
        }
    }
}
pub struct TileLoadEvent(pub u32);
fn updateTileLoadProgress_system(
    mut tile_quad_tree: ResMut<TileQuadTree>,
    queue_tile_to_render: Query<(Entity, &TileKey), With<TileToRender>>,
    queue_tile_to_update_height: Query<Entity, With<TileToUpdateHeight>>,
    queue_tile_load_high: Query<Entity, With<TileLoadHigh>>,
    queue_tile_load_medium: Query<Entity, With<TileLoadMedium>>,
    queue_tile_load_low: Query<Entity, With<TileLoadLow>>,
    mut tile_load_event_writer: EventWriter<TileLoadEvent>,
) {
    let p0_count = queue_tile_to_render.iter().count();
    let p1_count = queue_tile_to_update_height.iter().count();
    let p2_count = queue_tile_load_high.iter().count();
    let p3_count = queue_tile_load_medium.iter().count();
    let p4_count = queue_tile_load_low.iter().count();
    let currentLoadQueueLength = (p2_count + p3_count + p4_count) as u32;
    if tile_quad_tree._lastTileLoadQueueLength != currentLoadQueueLength
        || tile_quad_tree._tilesInvalidated
    {
        tile_quad_tree._lastTileLoadQueueLength = currentLoadQueueLength;
        tile_load_event_writer.send(TileLoadEvent(currentLoadQueueLength));
    }
    let debug = &mut tile_quad_tree.debug;
    if (debug.enableDebugOutput && !debug.suspendLodUpdate) {
        debug.maxDepth = queue_tile_to_render
            .iter()
            .map(|(entity, key)| key.level)
            .max()
            .unwrap_or(0);
        debug.tilesRendered = p0_count as u32;

        if (debug.tilesVisited != debug.lastTilesVisited
            || debug.tilesRendered != debug.lastTilesRendered
            || debug.tilesCulled != debug.lastTilesCulled
            || debug.maxDepth != debug.lastMaxDepth
            || debug.tilesWaitingForChildren != debug.lastTilesWaitingForChildren
            || debug.maxDepthVisited != debug.lastMaxDepthVisited)
        {
            println!("Visited {}, Rendered: {}, Culled: {}, Max Depth Rendered: {}, Max Depth Visited: {}, Waiting for children: {}",debug.tilesVisited,debug.tilesRendered,debug.tilesCulled,debug.maxDepth,debug.maxDepthVisited,debug.tilesWaitingForChildren);

            debug.lastTilesVisited = debug.tilesVisited;
            debug.lastTilesRendered = debug.tilesRendered;
            debug.lastTilesCulled = debug.tilesCulled;
            debug.lastMaxDepth = debug.maxDepth;
            debug.lastTilesWaitingForChildren = debug.tilesWaitingForChildren;
            debug.lastMaxDepthVisited = debug.maxDepthVisited;
        }
    }
}
fn quad_tile_state_end_system(
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
    mut imagery_layer_query: Query<(
        Entity,
        &mut Visibility,
        &mut ImageryLayer,
        &mut XYZDataSource,
    )>,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    mut datasource_query: Query<&mut TerrainDataSource>,
    mut asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    mut indicesAndEdgesCache: ResMut<IndicesAndEdgesCacheArc>,
    render_device: Res<RenderDevice>,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    let Ok(mut terrain_datasource) = datasource_query.get_single_mut()else{return;};
    for (
        entity,
        mut globe_surface_tile,
        rectangle,
        mut other_state,
        mut replacement_state,
        key,
        node_id,
        node_children,
        mut state,
        location,
        parent,
    ) in &mut quadtree_tile_query
    {
        let mut terrainOnly = if let Some(v) = globe_surface_tile.boundingVolumeSourceTile {
            v == entity
        } else {
            false
        } || other_state._lastSelectionResult
            == TileSelectionResult::CULLED_BUT_NEEDED;
        // let terrainStateBefore = globe_surface_tile.terrain_state.clone();

        if terrainOnly {
            return;
        }
        info!("quad_tile_state_end_system");

        let wasAlreadyRenderable = other_state.renderable;

        // The terrain is renderable as soon as we have a valid vertex array.
        other_state.renderable = globe_surface_tile.has_mesh();

        // But it's not done loading until it's in the READY state.
        let isTerrainDoneLoading = globe_surface_tile.terrain_state == TerrainState::READY;

        // If this tile's terrain and imagery are just upsampled from its parent, mark the tile as
        // upsampled only.  We won't refine a tile if its four children are upsampled only.
        other_state.upsampledFromParent = globe_surface_tile.terrainData.is_some()
            && globe_surface_tile
                .terrainData
                .as_ref()
                .expect("globe_surface_tile.terrainData")
                .lock()
                .expect("globe_surface_tile.terrainData.lock")
                .wasCreatedByUpsampling();

        let isImageryDoneLoading = {
            //GlobeSurfaceTile.prototype.processImagery
            let mut isUpsampledOnly = other_state.upsampledFromParent;
            let mut isAnyTileLoaded = false;
            let mut isDoneLoading = true;

            // Transition imagery states

            let mut length = globe_surface_tile.imagery.len();
            let mut i = 0;
            loop {
                if !(i >= 0 && i < length) {
                    break;
                }
                let tileImageryCollection = &mut globe_surface_tile.imagery;
                let mut tileImagery = tileImageryCollection.get_mut(i).expect("tilg_imagery");

                if tileImagery.loadingImagery.is_none() {
                    isUpsampledOnly = false;
                    continue;
                }
                let imagery_layer_entity = tileImagery.get_loading_imagery_layer_entity();
                let (_, _, mut imagery_layer, mut imagery_datasource) =
                    imagery_layer_query.get_mut(imagery_layer_entity).unwrap();
                let loading_imagery = tileImagery.get_loading_imagery(&imagery_layer).unwrap();
                // ImageryProvider.ready is deprecated. This is here for backwards compatibility

                if (loading_imagery.state == ImageryState::PLACEHOLDER) {
                    if (imagery_layer.ready && imagery_datasource.ready) {
                        // Remove the placeholder and add the actual skeletons (if any)
                        // at the same position.  Then continue the loop at the same index.
                        // tileImagery.freeResources();
                        tileImageryCollection.remove(i);
                        imagery_layer._createTileImagerySkeletons(
                            &mut globe_surface_tile,
                            rectangle,
                            key,
                            // &mut quadtree_tile_query,
                            // entity,
                            &mut terrain_datasource,
                            &mut imagery_datasource,
                            imagery_layer_entity,
                        );
                        i -= 1;
                        length = globe_surface_tile.imagery.len();
                        continue;
                    } else {
                        isUpsampledOnly = false;
                    }
                }

                let thisTileDoneLoading = tileImagery.processStateMachine(
                    false,
                    &mut imagery_layer,
                    &mut imagery_datasource,
                    rectangle,
                    &mut asset_server,
                    &mut images,
                    &mut render_world_queue,
                    &mut indicesAndEdgesCache,
                    &render_device,
                    &globe_camera,
                );
                isDoneLoading = isDoneLoading && thisTileDoneLoading;

                // The imagery is renderable as soon as we have any renderable imagery for this region.
                // 只要这块区域有一个可渲染的图片。imagery就是可渲染的。
                isAnyTileLoaded =
                    isAnyTileLoaded || thisTileDoneLoading || tileImagery.readyImagery.is_some();
                let loading_imagery = tileImagery.get_loading_imagery(&imagery_layer).unwrap();

                isUpsampledOnly = isUpsampledOnly
                    && (loading_imagery.state == ImageryState::FAILED
                        || loading_imagery.state == ImageryState::INVALID);
                i += 1;
            }

            other_state.upsampledFromParent = isUpsampledOnly;

            // Allow rendering if any available layers are loaded
            //如果任何可用图层加载上，就渲染
            other_state.renderable = other_state.renderable && (isAnyTileLoaded || isDoneLoading);

            isDoneLoading
        };

        if (isTerrainDoneLoading && isImageryDoneLoading) {
            *state = QuadtreeTileLoadState::DONE;
        }

        // Once a tile is renderable, it stays renderable, because doing otherwise would
        // cause detail (or maybe even the entire globe) to vanish when adding a new
        // imagery layer. `GlobeSurfaceTileProvider._onLayerAdded` sets renderable to
        // false for all affected tiles that are not currently being rendered.
        if (wasAlreadyRenderable) {
            other_state.renderable = true;
        }
        // if terrainOnly && terrainStateBefore != globe_surface_tile.terrain_state {
        //     if computeTileVisibility(
        //         // commands,
        //         // ellipsoid,
        //         &tile_quad_tree._occluders,
        //         &mut quadtree_tile_query,
        //         &mut globe_camera,
        //         entity,
        //     ) != TileVisibility::NONE
        //         && if let Some(v) = globe_surface_tile.boundingVolumeSourceTile {
        //             v == entity
        //         } else {
        //             false
        //         }
        //     {
        //         terrainOnly = false;

        //     }
        // }
    }
}
fn quadtree_tile_load_state_done_system(
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
    mut commands: Commands,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (
        entity,
        mut globe_surface_tile,
        rectangle,
        mut other_state,
        mut replacement_state,
        key,
        node_id,
        node_children,
        mut state,
        location,
        parent,
    ) in &mut quadtree_tile_query
    {
        if *state == QuadtreeTileLoadState::DONE {
            info!("render tile key={:?}", key);
            let mut rng = rand::thread_rng();
            let r: f32 = rng.gen();
            let g: f32 = rng.gen();
            let b: f32 = rng.gen();
            commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(globe_surface_tile.get_mesh().unwrap()),
                material: terrain_materials.add(TerrainMeshMaterial {
                    color: Color::rgba(r, g, b, 1.0),
                    image: Some(asset_server.load("icon.png")),
                    // image: asset_server.load(format!("https://t5.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={}&TILEROW={}&TILEMATRIX={}&tk=b931d6faa76fc3fbe622bddd6522e57b",x,y,level)),
                    // image: asset_server.load(format!("tile/{}/{}/{}.png", level, y, x,)),
                    // image: Some(asset_server.load(format!(
                    //     "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
                    //     key.level, key.x, key.y,
                    // ))),
                    // image: None,
                }),
                // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
                ..Default::default()
            });
            commands.entity(entity).remove::<TileToLoad>();
        }
    }
}
fn quad_tile_state_init_system(
    mut imagery_layer_query: Query<(
        Entity,
        &mut Visibility,
        &mut ImageryLayer,
        &mut XYZDataSource,
    )>,
    mut datasource_query: Query<&mut TerrainDataSource>,
    // mut query: Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
    query_immut: Query<(Entity, &Rectangle, &TileKey, &QuadtreeTileParent), With<TileToLoad>>,
    mut globe_surface_tile_query: Query<(&mut GlobeSurfaceTile), With<TileToLoad>>,
    mut state_query: Query<(&mut QuadtreeTileLoadState), With<TileToLoad>>,
) {
    let Ok(mut terrain_datasource) = datasource_query.get_single_mut()else{return;};
    for (entity, rectangle, key, parent) in query_immut.iter() {
        let state = state_query
            .get_component::<QuadtreeTileLoadState>(entity)
            .unwrap();
        if *state == QuadtreeTileLoadState::START {
            //prepare new
            let mut available = terrain_datasource.getTileDataAvailable(key);
            if !available.is_none() && parent.0 != TileNode::None {
                if let TileNode::Internal(e) = parent.0 {
                    let parentKey = {
                        let key = query_immut.get_component::<TileKey>(e).unwrap();
                        key.clone()
                    };
                    let parentSurfaceTile =
                        globe_surface_tile_query.get_component_mut::<GlobeSurfaceTile>(e);
                    if parentSurfaceTile.is_ok() {
                        // let parentKey = quadtree_tile_query.get_component::<TileKey>(e).unwrap();
                        let parentSurfaceTile = parentSurfaceTile.unwrap();
                        if parentSurfaceTile.terrainData.is_some() {
                            available = Some(
                                parentSurfaceTile
                                    .terrainData
                                    .as_ref()
                                    .unwrap()
                                    .lock()
                                    .unwrap()
                                    .isChildAvailable(parentKey.x, parentKey.y, key.x, key.y),
                            );
                        }
                    }
                }
            }

            if let Some(v) = available {
                if v == false {
                    let mut globe_surface_tile = globe_surface_tile_query
                        .get_component_mut::<GlobeSurfaceTile>(entity)
                        .expect("entity have GlobeSurfaceTile component");
                    globe_surface_tile.terrain_state = TerrainState::FAILED;
                }
            }

            // // Map imagery tiles to this terrain tile
            for (imagery_layer_entity, visibility, mut imagery_layer, mut xyz_datasource) in
                &mut imagery_layer_query
            {
                let mut globe_surface_tile = globe_surface_tile_query
                    .get_component_mut::<GlobeSurfaceTile>(entity)
                    .unwrap();
                if let Visibility::Visible = *visibility {
                    imagery_layer._createTileImagerySkeletons(
                        &mut globe_surface_tile,
                        rectangle,
                        key,
                        &mut terrain_datasource,
                        &mut xyz_datasource,
                        imagery_layer_entity,
                    );
                }
            }
            let mut state = state_query
                .get_component_mut::<QuadtreeTileLoadState>(entity)
                .unwrap();
            *state = QuadtreeTileLoadState::LOADING;
        }
    }
}
fn terrain_state_machine_system(
    mut quadtree_tile_query: Query<
        (
            Entity,
            &QuadtreeTileParent,
            &mut QuadtreeTileLoadState,
            &TileKey,
        ),
        With<TileToLoad>,
    >,
    mut globe_surface_tile_query: Query<(&GlobeSurfaceTile), With<TileToLoad>>,
) {
    for (entity, parent, mut state, key) in &mut quadtree_tile_query {
        if *state == QuadtreeTileLoadState::LOADING {
            let globe_surface_tile = globe_surface_tile_query
                .get_component::<GlobeSurfaceTile>(entity)
                .unwrap();
            if globe_surface_tile.terrain_state == TerrainState::FAILED
                && parent.0 != TileNode::None
            {
                if let TileNode::Internal(v) = parent.0 {
                    let parent_globe_surface_tile = globe_surface_tile_query
                        .get_component_mut::<GlobeSurfaceTile>(v)
                        .unwrap();
                    let parentReady = parent_globe_surface_tile.terrainData.is_some()
                        && parent_globe_surface_tile
                            .terrainData
                            .as_ref()
                            .unwrap()
                            .lock()
                            .unwrap()
                            .canUpsample();
                    if (!parentReady) {
                        //TODO 在下一帧能为其父节点执行processStateMachine
                        // processStateMachine(
                        //     quadtree_tile_query,
                        //     v.clone(),
                        //     terrain_datasource,
                        //     commands,
                        //     imagery_layer_query,
                        //     task_executor,
                        //     indicesAndEdgesCache,
                        //     task_executor_create_mesh,
                        //     terrainOnly,
                        //     images,
                        //     render_world_queue,
                        //     asset_server,
                        // );
                    }
                }
            }
        }
    }
}
fn upsample_system(
    mut datasource_query: Query<(&mut TerrainDataSource)>,
    mut job_spawner: JobSpawner,
    mut finished_jobs: FinishedJobs,
    mut query: Query<
        (
            Entity,
            &mut GlobeSurfaceTile,
            &QuadtreeTileParent,
            &mut QuadtreeTileLoadState,
            &TileKey,
        ),
        With<TileToLoad>,
    >,
) {
    let Ok(terrain_datasource) = datasource_query.get_single() else {
        return;
    };
    // let mut quadtree_tile_query = params_set.p0();
    for (entity, globe_surface_tile, parent, mut state, key) in query.iter() {
        if globe_surface_tile.terrain_state == TerrainState::FAILED {
            if let TileNode::None = parent.0 {
                let mut state = query
                    .get_component_mut::<QuadtreeTileLoadState>(entity)
                    .unwrap();
                *state = QuadtreeTileLoadState::FAILED;
                return;
            }

            if let TileNode::Internal(v) = parent.0 {
                let (terrain_data, parent_key) = {
                    // let mut world = params_set.p1();
                    let parent_globe_surface_tile =
                        query.get_component::<GlobeSurfaceTile>(v).unwrap();
                    if parent_globe_surface_tile.terrainData.is_none() {
                        continue;
                    }
                    let parent_key = query.get_component::<TileKey>(v).unwrap();
                    (
                        parent_globe_surface_tile
                            .terrainData
                            .as_ref()
                            .unwrap()
                            .clone(),
                        parent_key.clone(),
                    )
                };

                job_spawner.spawn(UpsampleJob {
                    terrain_data: terrain_data,
                    tiling_scheme: terrain_datasource.tiling_scheme.clone(),
                    parent_key: parent_key,
                    key: key.clone(),
                    entity: entity,
                });
                // globe_surface_tile.terrain_state = TerrainState::RECEIVING;
            }
        }
    }
    while let Some(result) = finished_jobs.take_next::<UpsampleJob>() {
        if let Ok(res) = result {
            let mut globe_surface_tile = query
                .get_component_mut::<GlobeSurfaceTile>(res.entity)
                .unwrap();
            if let Some(new_terrain_data) = res.terrain_data {
                globe_surface_tile.terrainData = Some(Arc::new(Mutex::new(new_terrain_data)));
                globe_surface_tile.terrain_state = TerrainState::RECEIVED;
            } else {
                globe_surface_tile.terrain_state = TerrainState::FAILED;
            }
        }
    }
}
fn request_tile_geometry_system(
    mut quadtree_tile_query: Query<(&TileKey, &mut GlobeSurfaceTile), With<TileToLoad>>,
    mut datasource_query: Query<&mut TerrainDataSource>,
) {
    let Ok(terrain_datasource) = datasource_query.get_single()else{return;};
    for (key, mut globe_surface_tile) in &mut quadtree_tile_query {
        if globe_surface_tile.terrain_state == TerrainState::UNLOADED {
            globe_surface_tile.terrain_state = TerrainState::RECEIVING;
            let value = terrain_datasource
                .requestTileGeometry()
                .expect("terrain_datasource.requestTileGeometry");
            globe_surface_tile.terrainData = Some(Arc::new(Mutex::new(value)));
            globe_surface_tile.terrain_state = TerrainState::RECEIVED;
        }
    }
}

fn transform_system(
    mut quadtree_tile_query: Query<(Entity, &TileKey, &mut GlobeSurfaceTile), With<TileToLoad>>,
    mut datasource_query: Query<&mut TerrainDataSource>,
    indicesAndEdgesCache: Res<IndicesAndEdgesCacheArc>,
    mut job_spawner: JobSpawner,
    mut finished_jobs: FinishedJobs,
) {
    let Ok(terrain_datasource) = datasource_query.get_single()else{return;};
    for (entity, key, mut globe_surface_tile) in &mut quadtree_tile_query {
        if globe_surface_tile.terrain_state == TerrainState::RECEIVED {
            job_spawner.spawn(CreateTileJob {
                terrain_data: globe_surface_tile.terrainData.as_ref().unwrap().clone(),
                key: key.clone(),
                tiling_scheme: terrain_datasource.tiling_scheme.clone(),
                indicesAndEdgesCache: indicesAndEdgesCache.get_cloned_cache(),
                entity: entity,
            });
            globe_surface_tile.terrain_state = TerrainState::TRANSFORMING;
        }
        if globe_surface_tile.terrain_state == TerrainState::TRANSFORMED {
            globe_surface_tile.terrain_state = TerrainState::READY;
        }
        if globe_surface_tile.terrain_state == TerrainState::READY {}
    }
    while let Some(result) = finished_jobs.take_next::<CreateTileJob>() {
        if let Ok(res) = result {
            let mut globe_surface_tile = quadtree_tile_query
                .get_component_mut::<GlobeSurfaceTile>(res.entity)
                .unwrap();
            globe_surface_tile.terrain_state = TerrainState::TRANSFORMED;
        }
    }
}
