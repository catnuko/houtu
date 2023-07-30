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
use crate::plugins::tileset::quadtree_tile::TileRendered;
use crate::plugins::tileset::terrain_datasource::TerrainDataSourceState;

use super::globe_surface_tile::{
    self, computeTileVisibility, GlobeSurfaceTile, TerrainState, TileVisibility,
};
use super::imagery::{Imagery, ImageryState};
use super::imagery_layer::{self, ImageryLayer, ImageryLayerOtherState};
use super::indices_and_edges_cache::IndicesAndEdgesCacheArc;
use super::reproject_texture::{self, ReprojectTextureTaskQueue};
use super::terrain_datasource::{self, TerrainDataSource, TerrainDataSourceData};
use super::terrian_material::TerrainMeshMaterial;
use super::tile_selection_result::TileSelectionResult;
use super::traversal_details::{
    get_traversal_details, AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails,
};
use super::visit_visible_children_near_to_far::{
    self, make_new_quadtree_tile, subdivide, visit_if_visible, visit_visible_children_near_to_far,
};
use super::xyz_datasource::XYZDataSource;
use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileData, QuadtreeTileLoadState,
    QuadtreeTileMark, QuadtreeTileOtherState, QuadtreeTileParent, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileRenderedToDestroy, TileToLoad, TileToRender, TileToUpdateHeight,
};
use super::tile_replacement_queue::{TileReplacementQueue, TileReplacementState};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(imagery_layer::Plugin);
        app.add_plugin(terrain_datasource::Plugin);
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
        app.add_system(end_frame.before(update_tile_load_progress_system));
        app.add_system(update_tile_load_progress_system);
        app.add_systems((quad_tile_state_init_system, quad_tile_state_end_system));
        app.add_system(ImageryLayer::finish_reproject_texture_system);
        app.add_system(quadtree_tile_load_state_done_system);
    }
}

#[derive(Resource)]
pub struct TileQuadTree {
    pub tile_cache_size: u32,
    pub maximum_screen_space_error: f64,
    pub load_queue_time_slice: u32,
    pub loading_descendant_limit: u32,
    pub preload_ancestors: bool,
    pub preload_siblings: bool,
    pub tiles_invalidated: bool,
    pub last_tile_load_queue_length: u32,
    pub last_selection_frame_number: Option<u32>,
    pub occluders: EllipsoidalOccluder,
    pub camera_position_cartographic: Option<Cartographic>,
    pub camera_reference_frame_origin_cartographic: Option<Cartographic>,
    pub replacement_queue: TileReplacementQueue,
    pub debug: TileQuadTreeDebug,
}

impl TileQuadTree {
    pub fn new() -> Self {
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
            occluders: EllipsoidalOccluder::default(),
            camera_position_cartographic: None,
            camera_reference_frame_origin_cartographic: None,
            replacement_queue: TileReplacementQueue::new(),
            debug: TileQuadTreeDebug::new(),
        }
    }
    /// 调用后将清空所有瓦片重新创建
    pub fn invalidate_all_tiles(&mut self) {
        self.tiles_invalidated = true;
    }
    pub fn real_invalidateAllTiles(&mut self) {}
}
pub struct TileQuadTreeDebug {
    pub enable_debug_output: bool,

    pub max_depth: u32,
    pub max_depth_visited: u32,
    pub tiles_visited: u32,
    pub tiles_culled: u32,
    pub tiles_rendered: u32,
    pub tiles_waiting_for_children: u32,

    pub last_max_depth: u32,
    pub last_max_depth_visited: u32,
    pub last_tiles_visited: u32,
    pub last_tiles_culled: u32,
    pub last_tiles_rendered: u32,
    pub last_tiles_waiting_for_children: u32,

    pub suspend_lod_update: bool,
}
impl TileQuadTreeDebug {
    pub fn new() -> Self {
        Self {
            enable_debug_output: true,
            max_depth: 0,
            max_depth_visited: 0,
            tiles_visited: 0,
            tiles_culled: 0,
            tiles_rendered: 0,
            tiles_waiting_for_children: 0,
            last_max_depth: 0,
            last_max_depth_visited: 0,
            last_tiles_visited: 0,
            last_tiles_culled: 0,
            last_tiles_rendered: 0,
            last_tiles_waiting_for_children: 0,
            suspend_lod_update: false,
        }
    }
    pub fn reset(&mut self) {
        self.max_depth = 0;
        self.max_depth_visited = 0;
        self.tiles_visited = 0;
        self.tiles_culled = 0;
        self.tiles_rendered = 0;
        self.tiles_waiting_for_children = 0;
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
    if tile_quad_tree.tiles_invalidated {
        tile_quad_tree.real_invalidateAllTiles();
        tile_quad_tree.tiles_invalidated = false;
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
    if tile_quad_tree.debug.suspend_lod_update {
        return;
    }
    tile_quad_tree.replacement_queue.markStartOfRenderFrame();
    // TODO createRenderCommandsForSelectedTiles函数开始
}
fn render(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    render_queue_query: Query<Entity, With<TileToRender>>,
    rendered_query: Query<(Entity, &mut TileRendered)>,
    mut terrain_datasource: ResMut<TerrainDataSource>,
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery>,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
    ellipsoidal_occluder: Res<EllipsoidalOccluder>,
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
    if tile_quad_tree.debug.suspend_lod_update {
        return;
    }
    //清空渲染列表
    render_queue_query.iter().for_each(|entity: Entity| {
        let mut entity_mut = commands.get_entity(entity).expect("entity不存在");
        entity_mut.remove::<TileToRender>();
    });
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
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
            let number_of_level_zero_tiles_x = terrain_datasource
                .tiling_scheme
                .get_number_of_x_tiles_at_level(0);
            let number_of_level_zero_tiles_y = terrain_datasource
                .tiling_scheme
                .get_number_of_y_tiles_at_level(0);
            let mut i = 0;
            for y in 0..number_of_level_zero_tiles_y {
                for x in 0..number_of_level_zero_tiles_x {
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
    let camera_frame_origin = globe_camera.get_transform().get_translation();
    tile_quad_tree.camera_position_cartographic = Some(p.clone());
    tile_quad_tree.camera_reference_frame_origin_cartographic =
        Ellipsoid::WGS84.cartesianToCartographic(&camera_frame_origin);
    tt.iter().enumerate().for_each(|(_, x)| {
        let (entity, _) = x;
        tile_quad_tree
            .replacement_queue
            .mark_tile_rendered(&mut quadtree_tile_query, *entity);
        let mut other_state = quadtree_tile_query
            .get_component_mut::<QuadtreeTileOtherState>(*entity)
            .unwrap();
        if !other_state.renderable {
            commands.entity(*entity).insert(TileLoadHigh);
            tile_quad_tree.debug.tiles_waiting_for_children += 1;
        } else {
            let mut ancestor_meets_sse = false;
            visit_if_visible(
                &mut commands,
                &mut tile_quad_tree,
                &ellipsoidal_occluder.ellipsoid,
                &ellipsoidal_occluder,
                &mut quadtree_tile_query,
                &frame_count,
                &mut globe_camera,
                window,
                terrain_datasource.as_mut(),
                &mut ancestor_meets_sse,
                &mut all_traversal_quad_details,
                &mut root_traversal_details,
                &mut queue_params_set,
                *entity,
            );
        }
    });
    //清除已经渲染但不需要渲染的瓦片
    rendered_query.iter().for_each(|(e, tile_rendered)| {
        if !render_queue_query.contains(e) {
            commands.entity(tile_rendered.0).despawn();
        }
    });
}

pub fn visitTile(
    commands: &mut Commands,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
    ellipsoid: &Ellipsoid,
    ellipsoidal_occluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    frame_count: &Res<FrameCount>,
    globe_camera: &mut GlobeCamera,
    window: &Window,
    terrain_datasource: &mut TerrainDataSource,
    ancestor_meets_sse: &mut bool,
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
    tile_quad_tree.debug.tiles_visited += 1;

    tile_quad_tree
        .replacement_queue
        .mark_tile_rendered(quadtree_tile_query, quadtree_tile_entity);
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
        terrain_datasource_data,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    // info!("visitTile key={:?}", key);
    if key.level > tile_quad_tree.debug.max_depth_visited {
        tile_quad_tree.debug.max_depth_visited = key.level;
    }
    let traversal_details = get_traversal_details(
        all_traversal_quad_details,
        root_traversal_details,
        location,
        key,
    );
    let mut entity_mut = commands.entity(entity);

    let meets_sse = screen_space_error(
        key,
        &mut other_state,
        globe_camera,
        window,
        ellipsoid,
        terrain_datasource,
    ) < tile_quad_tree.maximum_screen_space_error;
    subdivide(
        entity_mut.commands(),
        node_id,
        key,
        &mut node_children,
        terrain_datasource,
    );
    let south_west_child = node_children.southwest;
    let south_east_child = node_children.southeast;
    let north_west_child = node_children.northwest;
    let north_east_child = node_children.northeast;

    let last_frame = tile_quad_tree.last_selection_frame_number;
    let last_frame_selection_result = if other_state.last_selection_result_frame == last_frame {
        other_state.last_selection_result.clone()
    } else {
        TileSelectionResult::NONE
    };
    if meets_sse || *ancestor_meets_sse {
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
        let one_rendered_last_frame =
            TileSelectionResult::originalResult(&last_frame_selection_result)
                == TileSelectionResult::RENDERED as u8;
        let two_culled_or_not_visited =
            TileSelectionResult::originalResult(&last_frame_selection_result)
                == TileSelectionResult::CULLED as u8
                || last_frame_selection_result == TileSelectionResult::NONE;
        let three_completely_loaded = *state == QuadtreeTileLoadState::DONE;

        let mut renderable =
            one_rendered_last_frame || two_culled_or_not_visited || three_completely_loaded;

        if !renderable {
            // Check the more expensive condition 4 above. This requires details of the thing
            // we're rendering (e.g. the globe surface), so delegate it to the tile provider.
            /*
            上面四个条件满足其一时可渲染
            */
            renderable = false
        }

        if renderable {
            // Only load this tile if it (not just an ancestor) meets the SSE.
            // 只有当前瓦片满足SSE时再去加载所需资源
            if meets_sse {
                entity_mut.insert(TileLoadMedium);
            }
            entity_mut.insert(TileToRender);
            entity_mut.remove::<(TileToLoad)>();

            traversal_details.all_are_renderable = other_state.renderable;
            traversal_details.any_were_rendered_last_frame =
                last_frame_selection_result == TileSelectionResult::RENDERED;
            traversal_details.not_yet_renderable_count = if other_state.renderable { 0 } else { 1 };

            other_state.last_selection_result_frame = Some(frame_count.0);
            other_state.last_selection_result = TileSelectionResult::RENDERED;

            if !traversal_details.any_were_rendered_last_frame {
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
        *ancestor_meets_sse = true;

        // Load this blocker tile with high priority, but only if this tile (not just an ancestor) meets the SSE.
        if meets_sse {
            entity_mut.insert(TileLoadHigh);
        }
    }

    if terrain_datasource.can_refine(&terrain_datasource_data, key) {
        let mut all_are_upsampled = true;
        if let TileNode::Internal(v) = south_west_child {
            if let Ok(state) = quadtree_tile_query.get_component::<QuadtreeTileOtherState>(v) {
                all_are_upsampled = all_are_upsampled && state.upsampled_from_parent;
            } else {
                return;
            }
        }
        if let TileNode::Internal(v) = south_east_child {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            all_are_upsampled = all_are_upsampled && state.upsampled_from_parent;
        }
        if let TileNode::Internal(v) = north_west_child {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            all_are_upsampled = all_are_upsampled && state.upsampled_from_parent;
        }
        if let TileNode::Internal(v) = north_east_child {
            let state = quadtree_tile_query
                .get_component::<QuadtreeTileOtherState>(v)
                .unwrap();
            all_are_upsampled = all_are_upsampled && state.upsampled_from_parent;
        }

        if all_are_upsampled {
            // No point in rendering the children because they're all upsampled.  Render this tile instead.
            // 没必要渲染子孙瓦片，因为他们都是被采样过的。渲染当前瓦片。
            entity_mut.insert(TileToRender);

            // Rendered tile that's not waiting on children loads with medium priority.
            // 渲染瓦片，不等待中等优先级的子孙瓦片加载
            entity_mut.insert(TileLoadHigh);

            // Make sure we don't unload the children and forget they're upsampled.
            // 确保我们没卸载子孙瓦片忘记他们被采样过。
            let mut mark_tile_rendered_child =
                |tile_quad_tree: &mut ResMut<TileQuadTree>,
                 node_id: &TileNode,
                 quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>| {
                    if let TileNode::Internal(v) = node_id {
                        tile_quad_tree
                            .replacement_queue
                            .mark_tile_rendered(quadtree_tile_query, v.clone());
                    }
                };
            mark_tile_rendered_child(tile_quad_tree, &south_west_child, quadtree_tile_query);
            mark_tile_rendered_child(tile_quad_tree, &south_east_child, quadtree_tile_query);
            mark_tile_rendered_child(tile_quad_tree, &north_west_child, quadtree_tile_query);
            mark_tile_rendered_child(tile_quad_tree, &north_east_child, quadtree_tile_query);
            let mut other_state = quadtree_tile_query
                .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
                .unwrap();
            traversal_details.all_are_renderable = other_state.renderable;
            traversal_details.any_were_rendered_last_frame =
                last_frame_selection_result == TileSelectionResult::RENDERED;
            traversal_details.not_yet_renderable_count = if other_state.renderable { 0 } else { 1 };

            other_state.last_selection_result_frame = Some(frame_count.0);
            other_state.last_selection_result = TileSelectionResult::RENDERED;

            if !traversal_details.any_were_rendered_last_frame {
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
        other_state.last_selection_result_frame = Some(frame_count.0);
        other_state.last_selection_result = TileSelectionResult::REFINED;

        let first_rendered_descendant_index = queue_params_set.p0().iter().count();
        let load_index_low = queue_params_set.p4().iter().count();
        let load_index_medium = queue_params_set.p3().iter().count();
        let load_index_high = queue_params_set.p2().iter().count();
        let tiles_to_update_heights_index = queue_params_set.p1().iter().count();

        // No need to add the children to the load queue because they'll be added (if necessary) when they're visited.
        // 不需要将子孙放入加载队列，因为他们被visited时会被加载
        visit_visible_children_near_to_far(
            entity_mut.commands(),
            tile_quad_tree,
            ellipsoid,
            ellipsoidal_occluder,
            quadtree_tile_query,
            frame_count,
            globe_camera,
            window,
            terrain_datasource,
            ancestor_meets_sse,
            queue_params_set,
            all_traversal_quad_details,
            root_traversal_details,
            &south_west_child,
            &south_east_child,
            &north_west_child,
            &north_east_child,
            quadtree_tile_entity,
        );
        let key = quadtree_tile_query
            .get_component::<TileKey>(quadtree_tile_entity)
            .unwrap();
        let location = quadtree_tile_query
            .get_component::<Quadrant>(quadtree_tile_entity)
            .unwrap();
        let traversal_details = get_traversal_details(
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

        if first_rendered_descendant_index != render_count {
            // At least one descendant tile was added to the render list.
            // The traversal_details tell us what happened while visiting the children.
            /*
            至少一个子孙瓦片被添加到渲染列表中。traversalDetails告诉我们遍历子孙时发生了什么。
             */

            let all_are_renderable = traversal_details.all_are_renderable;
            let any_were_rendered_last_frame = traversal_details.any_were_rendered_last_frame;
            let not_yet_renderable_count = traversal_details.not_yet_renderable_count;
            let mut queued_for_load = false;
            /*
            如果all_are_renderable==false，意味着当前瓦片和子孙瓦片中有一个瓦片不可被渲染
            如果all_are_renderable==True，意味着当前瓦片和子孙瓦片都可渲染
            如果any_were_rendered_last_frame==false，意味着当前瓦片和子孙瓦片的上帧渲染结果都不等于Rendered
            如果any_were_rendered_last_frame==True，意味着当前瓦片和子孙瓦片的上帧渲染结果中有一个等于Rendered
            如果它俩都等于false，则意味着当前瓦片和子孙瓦片上帧都没被渲染过，而且有一个子孙瓦片不可被渲染
            此时，执行下面的操作，将所有子孙瓦片从渲染列表中踢出，只渲染当前瓦片，继续加载子孙瓦片。
            */
            if !all_are_renderable && !any_were_rendered_last_frame {
                // Some of our descendants aren't ready to render yet, and none were rendered last frame,
                // so kick them all out of the render list and render this tile instead. Continue to load them though!

                // Mark the rendered descendants and their ancestors - up to this tile - as kicked.
                /*
                子孙瓦片中的一些还没准备好渲染而且没有一个在上一帧渲染了，所以将所有子孙瓦片从渲染列表中踢出去
                只渲染当前瓦片，继续加载子孙瓦片。

                标记被渲染的子孙瓦片和他们的祖先瓦片(直到当前瓦片),然后踢出去
                 */
                queue_params_set.p0().iter().enumerate().for_each(|(i, e)| {
                    if i >= first_rendered_descendant_index {
                        let mut work_tile_a = e.clone();
                        while (work_tile_a != entity) {
                            let mut work_tile = quadtree_tile_query.get_mut(work_tile_a).unwrap();
                            let other_state = &mut work_tile.3;
                            other_state.last_selection_result = TileSelectionResult::from_u8(
                                TileSelectionResult::kick(&other_state.last_selection_result),
                            );
                            let parent = &work_tile.10;
                            if let QuadtreeTileParent(TileNode::Internal(v)) = parent {
                                work_tile_a = v.clone();
                            }
                        }
                    }
                });

                // Remove all descendants from the render list and add this tile.
                // 移除所有的子孙瓦片和当前瓦片
                remove_component(
                    entity_mut.commands(),
                    &queue_params_set.p0(),
                    first_rendered_descendant_index,
                );

                remove_component(
                    entity_mut.commands(),
                    &queue_params_set.p1(),
                    tiles_to_update_heights_index,
                );
                entity_mut.insert(TileToRender);

                let mut other_state = quadtree_tile_query
                    .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
                    .unwrap();
                other_state.last_selection_result = TileSelectionResult::RENDERED;

                // If we're waiting on heaps of descendants, the above will take too long. So in that case,
                // load this tile INSTEAD of loading any of the descendants, and tell the up-level we're only waiting
                // on this tile. Keep doing this until we actually manage to render this tile.
                /*
                如果我们要等一大堆后代，上面的计算将花费很长时间，所以这种情况，加载当前瓦片而不是任何子孙瓦片，并告诉上层？
                我们正在等待当前瓦片加载，继续这样做直到我们能渲染当前瓦片
                 */
                let was_rendered_last_frame =
                    last_frame_selection_result == TileSelectionResult::RENDERED;
                if !was_rendered_last_frame
                    && not_yet_renderable_count > tile_quad_tree.loading_descendant_limit
                {
                    // Remove all descendants from the load queues.
                    remove_component(
                        entity_mut.commands(),
                        &queue_params_set.p4(),
                        load_index_low,
                    );
                    remove_component(
                        entity_mut.commands(),
                        &queue_params_set.p3(),
                        load_index_medium,
                    );
                    remove_component(
                        entity_mut.commands(),
                        &queue_params_set.p2(),
                        load_index_high,
                    );
                    entity_mut.insert(TileLoadMedium);
                    traversal_details.not_yet_renderable_count =
                        if other_state.renderable { 0 } else { 1 };
                    queued_for_load = true;
                }

                traversal_details.all_are_renderable = other_state.renderable;
                traversal_details.any_were_rendered_last_frame = was_rendered_last_frame;

                if !was_rendered_last_frame {
                    // Tile is newly-rendered this frame, so update its heights.
                    // 瓦片时这帧刚渲染的，所以更新它的高度
                    entity_mut.insert(TileToUpdateHeight);
                }
                tile_quad_tree.debug.tiles_waiting_for_children += 1;
            }

            if tile_quad_tree.preload_ancestors && !queued_for_load {
                entity_mut.insert(TileLoadLow);
            }
        }

        return;
    }
    let renderable = {
        let mut other_state = quadtree_tile_query
            .get_component_mut::<QuadtreeTileOtherState>(quadtree_tile_entity)
            .unwrap();
        other_state.last_selection_result_frame = Some(frame_count.0);
        other_state.last_selection_result = TileSelectionResult::RENDERED;
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
    traversal_details.all_are_renderable = renderable;
    traversal_details.any_were_rendered_last_frame =
        last_frame_selection_result == TileSelectionResult::RENDERED;
    traversal_details.not_yet_renderable_count = if renderable { 0 } else { 1 };
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
    &'a TerrainDataSourceData,
);

fn screen_space_error(
    key: &TileKey,
    other_state: &QuadtreeTileOtherState,
    globe_camera: &GlobeCamera,
    window: &Window,
    ellipsoid: &Ellipsoid,
    terrain_datasource: &mut TerrainDataSource,
) -> f64 {
    let max_geometric_error: f64 = terrain_datasource.get_level_maximum_geometric_error(key.level);

    let distance = other_state._distance;
    let height = window.height() as f64;
    let sse_denominator = globe_camera.frustum.sse_denominator();

    let mut error = (max_geometric_error * height) / (distance * sse_denominator);

    error /= window.scale_factor();
    return error;
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
    process_tile_load_queue(
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

fn process_tile_load_queue(
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
    let size = tile_quad_tree.tile_cache_size;
    tile_quad_tree
        .replacement_queue
        .trimTiles(size, quadtree_tile_query);

    let end_time = frame_count.0 + tile_quad_tree.load_queue_time_slice;

    let mut did_some_loading = false;
    process_single_priority_load_queue(
        frame_count,
        commands,
        end_time,
        &queue_tile_load_high,
        &mut did_some_loading,
        quadtree_tile_query,
        tile_quad_tree,
    );
    process_single_priority_load_queue(
        frame_count,
        commands,
        end_time,
        &queue_tile_load_medium,
        &mut did_some_loading,
        quadtree_tile_query,
        tile_quad_tree,
    );
    process_single_priority_load_queue(
        frame_count,
        commands,
        end_time,
        &queue_tile_load_low,
        &mut did_some_loading,
        quadtree_tile_query,
        tile_quad_tree,
    );
}

fn process_single_priority_load_queue<T: Component>(
    frame_count: &Res<FrameCount>,
    commands: &mut Commands,
    end_time: u32,
    load_queue: &Query<Entity, With<T>>,
    did_some_loading: &mut bool,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    tile_quad_tree: &mut ResMut<TileQuadTree>,
) {
    for i in load_queue.iter() {
        tile_quad_tree
            .replacement_queue
            .mark_tile_rendered(quadtree_tile_query, i);
        commands.entity(i).insert(TileToLoad);
        *did_some_loading = true;
        if !(frame_count.0 < end_time || !*did_some_loading) {
            break;
        }
    }
}
pub struct TileLoadEvent(pub u32);
fn update_tile_load_progress_system(
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
    let current_load_queue_length = (p2_count + p3_count + p4_count) as u32;
    if tile_quad_tree.last_tile_load_queue_length != current_load_queue_length
        || tile_quad_tree.tiles_invalidated
    {
        tile_quad_tree.last_tile_load_queue_length = current_load_queue_length;
        tile_load_event_writer.send(TileLoadEvent(current_load_queue_length));
    }
    let debug = &mut tile_quad_tree.debug;
    if debug.enable_debug_output && !debug.suspend_lod_update {
        debug.max_depth = queue_tile_to_render
            .iter()
            .map(|(entity, key)| key.level)
            .max()
            .unwrap_or(0);
        debug.tiles_rendered = p0_count as u32;

        if (debug.tiles_visited != debug.last_tiles_visited
            || debug.tiles_rendered != debug.last_tiles_rendered
            || debug.tiles_culled != debug.last_tiles_culled
            || debug.max_depth != debug.last_max_depth
            || debug.tiles_waiting_for_children != debug.last_tiles_waiting_for_children
            || debug.max_depth_visited != debug.last_max_depth_visited)
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
fn quad_tile_state_end_system(
    mut quadtree_tile_query: Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
    mut imagery_layer_query: Query<(
        Entity,
        &mut Visibility,
        &mut ImageryLayer,
        &mut XYZDataSource,
    )>,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    mut terrain_datasource: ResMut<TerrainDataSource>,
    mut asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    mut indices_and_edges_cache: ResMut<IndicesAndEdgesCacheArc>,
    render_device: Res<RenderDevice>,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
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
        terrain_datasource_data,
    ) in &mut quadtree_tile_query
    {
        let mut terrainOnly = if let Some(v) = globe_surface_tile.bounding_volume_source_tile {
            v == entity
        } else {
            false
        } || other_state.last_selection_result
            == TileSelectionResult::CULLED_BUT_NEEDED;
        // let terrainStateBefore = terrain_datasource_data.state.clone();

        if terrainOnly {
            return;
        }
        info!("quad_tile_state_end_system");

        let was_already_renderable = other_state.renderable;

        // The terrain is renderable as soon as we have a valid vertex array.
        other_state.renderable = terrain_datasource_data.has_mesh();

        // But it's not done loading until it's in the READY state.
        let is_terrain_done_loading =
            terrain_datasource_data.state == TerrainDataSourceState::READY;

        // If this tile's terrain and imagery are just upsampled from its parent, mark the tile as
        // upsampled only.  We won't refine a tile if its four children are upsampled only.
        other_state.upsampled_from_parent = terrain_datasource_data.has_terrain_data()
            && terrain_datasource_data.was_created_by_upsampling();

        let is_imagery_done_loading = {
            //GlobeSurfaceTile.prototype.processImagery
            let mut is_upsampled_only = other_state.upsampled_from_parent;
            let mut is_any_tile_loaded = false;
            let mut is_done_loading = true;

            // Transition imagery states

            let mut length = globe_surface_tile.imagery.len();
            let mut i = 0;
            loop {
                if !(i >= 0 && i < length) {
                    break;
                }
                let tile_imagery_collection = &mut globe_surface_tile.imagery;
                let mut tile_imagery = tile_imagery_collection.get_mut(i).expect("tilg_imagery");

                if tile_imagery.loading_imagery.is_none() {
                    is_upsampled_only = false;
                    continue;
                }
                let imagery_layer_entity = tile_imagery.get_loading_imagery_layer_entity();
                let (_, _, mut imagery_layer, mut imagery_datasource) =
                    imagery_layer_query.get_mut(imagery_layer_entity).unwrap();
                let loading_imagery = tile_imagery.get_loading_imagery(&imagery_layer).unwrap();
                // ImageryProvider.ready is deprecated. This is here for backwards compatibility

                if loading_imagery.state == ImageryState::PLACEHOLDER {
                    if imagery_layer.ready && imagery_datasource.ready {
                        // Remove the placeholder and add the actual skeletons (if any)
                        // at the same position.  Then continue the loop at the same index.
                        // tile_imagery.freeResources();
                        tile_imagery_collection.remove(i);
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
                        is_upsampled_only = false;
                    }
                }

                let this_tile_done_loading = tile_imagery.process_state_machine(
                    false,
                    &mut imagery_layer,
                    &mut imagery_datasource,
                    rectangle,
                    &mut asset_server,
                    &mut images,
                    &mut render_world_queue,
                    &mut indices_and_edges_cache,
                    &render_device,
                    &globe_camera,
                );
                is_done_loading = is_done_loading && this_tile_done_loading;

                // The imagery is renderable as soon as we have any renderable imagery for this region.
                // 只要这块区域有一个可渲染的图片。imagery就是可渲染的。
                is_any_tile_loaded = is_any_tile_loaded
                    || this_tile_done_loading
                    || tile_imagery.ready_imagery.is_some();
                let loading_imagery = tile_imagery.get_loading_imagery(&imagery_layer).unwrap();

                is_upsampled_only = is_upsampled_only
                    && (loading_imagery.state == ImageryState::FAILED
                        || loading_imagery.state == ImageryState::INVALID);
                i += 1;
            }

            other_state.upsampled_from_parent = is_upsampled_only;

            // Allow rendering if any available layers are loaded
            //如果任何可用图层加载上，就渲染
            other_state.renderable =
                other_state.renderable && (is_any_tile_loaded || is_done_loading);

            is_done_loading
        };

        if is_terrain_done_loading && is_imagery_done_loading {
            *state = QuadtreeTileLoadState::DONE;
        }

        // Once a tile is renderable, it stays renderable, because doing otherwise would
        // cause detail (or maybe even the entire globe) to vanish when adding a new
        // imagery layer. `GlobeSurfaceTileProvider._onLayerAdded` sets renderable to
        // false for all affected tiles that are not currently being rendered.
        if was_already_renderable {
            other_state.renderable = true;
        }
        // if terrainOnly && terrainStateBefore != terrain_datasource_data.state {
        //     if computeTileVisibility(
        //         // commands,
        //         // ellipsoid,
        //         &tile_quad_tree.occluders,
        //         &mut quadtree_tile_query,
        //         &mut globe_camera,
        //         entity,
        //     ) != TileVisibility::NONE
        //         && if let Some(v) = globe_surface_tile.bounding_volume_source_tile {
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
    mut quadtree_tile_query: Query<
        GlobeSurfaceTileQuery,
        (With<TileToRender>, Without<TileRendered>),
    >,
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
        terrain_datasource_data,
    ) in &mut quadtree_tile_query
    {
        if *state == QuadtreeTileLoadState::DONE {
            info!("render tile key={:?}", key);
            let mut rng = rand::thread_rng();
            let r: f32 = rng.gen();
            let g: f32 = rng.gen();
            let b: f32 = rng.gen();
            let mut rendered_entity = commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(terrain_datasource_data.get_mesh().unwrap()),
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
            let entity = rendered_entity.id();
            rendered_entity
                .remove::<(TileToLoad, TileToRender)>()
                .insert(TileRendered(entity));
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
    mut terrain_datasource: ResMut<TerrainDataSource>,
    // mut query: Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
    query_immut: Query<(Entity, &Rectangle, &TileKey, &QuadtreeTileParent), With<TileToLoad>>,
    mut globe_surface_tile_query: Query<
        (&mut GlobeSurfaceTile, &mut TerrainDataSourceData),
        With<TileToLoad>,
    >,
    mut state_query: Query<(&mut QuadtreeTileLoadState), With<TileToLoad>>,
) {
    for (entity, rectangle, key, parent) in query_immut.iter() {
        let state = state_query
            .get_component::<QuadtreeTileLoadState>(entity)
            .unwrap();
        if *state == QuadtreeTileLoadState::START {
            //prepare new
            let mut available = terrain_datasource.get_tile_data_available(key);
            if !available.is_none() && parent.0 != TileNode::None {
                if let TileNode::Internal(e) = parent.0 {
                    let parent_key = {
                        let key = query_immut.get_component::<TileKey>(e).unwrap();
                        key.clone()
                    };
                    let parentSurfaceTile =
                        globe_surface_tile_query.get_component_mut::<TerrainDataSourceData>(e);
                    if parentSurfaceTile.is_ok() {
                        // let parent_key = quadtree_tile_query.get_component::<TileKey>(e).unwrap();
                        let parentSurfaceTile = parentSurfaceTile.unwrap();
                        if parentSurfaceTile.has_terrain_data() {
                            available =
                                Some(parentSurfaceTile.is_child_available(&parent_key, &key));
                        }
                    }
                }
            }

            if let Some(v) = available {
                if v == false {
                    let mut terrain_datasource_data = globe_surface_tile_query
                        .get_component_mut::<TerrainDataSourceData>(entity)
                        .expect("entity have GlobeSurfaceTile component");
                    terrain_datasource_data.state = TerrainDataSourceState::FAILED;
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
// fn terrain_state_machine_system(
//     mut quadtree_tile_query: Query<
//         (
//             Entity,
//             &QuadtreeTileParent,
//             &mut QuadtreeTileLoadState,
//             &TileKey,
//         ),
//         With<TileToLoad>,
//     >,
//     mut globe_surface_tile_query: Query<(&TerrainDataSourceData), With<TileToLoad>>,
// ) {
//     for (entity, parent, mut state, key) in &mut quadtree_tile_query {
//         if *state == QuadtreeTileLoadState::LOADING {
//             let terrain_datasource_data = globe_surface_tile_query
//                 .get_component::<TerrainDataSourceData>(entity)
//                 .unwrap();
//             if terrain_datasource_data.state == TerrainDataSourceState::FAILED
//                 && parent.0 != TileNode::None
//             {
//                 if let TileNode::Internal(v) = parent.0 {
//                     let parent_globe_surface_tile = globe_surface_tile_query
//                         .get_component_mut::<TerrainDataSourceData>(v)
//                         .unwrap();
//                     let parentReady = parent_globe_surface_tile.has_terrain_data()
//                         && parent_globe_surface_tile.can_upsample();
//                     if !parentReady {
//                         //TODO 在下一帧能为其父节点执行processStateMachine
//                         // process_state_machine(
//                         //     quadtree_tile_query,
//                         //     v.clone(),
//                         //     terrain_datasource,
//                         //     commands,
//                         //     imagery_layer_query,
//                         //     task_executor,
//                         //     indices_and_edges_cache,
//                         //     task_executor_create_mesh,
//                         //     terrainOnly,
//                         //     images,
//                         //     render_world_queue,
//                         //     asset_server,
//                         // );
//                     }
//                 }
//             }
//         }
//     }
// }
