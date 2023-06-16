use std::cmp::Ordering;
use std::sync::Arc;

use bevy::prelude::*;
use houtu_scene::{
    Cartographic, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Matrix4, Rectangle,
    TilingScheme,
};

use crate::plugins::camera::GlobeCamera;

use super::tile_tree::TileTree;

use super::quadtree_tile::{
    Quadrant, QuadtreeTile, QuadtreeTileMark, QuadtreeTileOtherState, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileToRender,
};
use super::tile_datasource::{self, QuadTreeTileDatasourceMark, Ready, TilingSchemeWrap};
use super::tile_replacement_queue::{
    self, EmitMarkTileRendered, TileReplacementQueue, TileReplacementState,
};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(tile_replacement_queue::Plugin);
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
    _lastSelectionFrameNumber: Option<f64>,
    _occluders: EllipsoidalOccluder,
    _cameraPositionCartographic: Option<Cartographic>,
    _cameraReferenceFrameOriginCartographic: Option<Cartographic>,
    tiletree: TileTree,
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
    mut tile_replacement_queue: ResMut<TileReplacementQueue>,
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
    tile_replacement_queue.markStartOfRenderFrame();

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
                    let mut entity_mut = commands.spawn(QuadtreeTile::new(
                        super::TileKey {
                            x: x,
                            y: y,
                            level: 0,
                        },
                        r,
                        Quadrant::Root,
                        TileNode::None,
                    ));
                    entity_mut.insert(TileReplacementState::new(entity_mut.id()));
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
    tile_quad_node_query.iter().for_each(|x| tt.push(x));
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
    for (entity, location, rectangle, replacement_state, other_state) in tt {
        let mut entity_mut = commands.entity(entity);
        entity_mut.insert(EmitMarkTileRendered);
        if !other_state.renderable {
        } else {
        }
    }
}
fn end_frame(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    mut tile_replacement_queue: ResMut<TileReplacementQueue>,
    high_queue_query: Query<Entity, With<TileLoadHigh>>,
    medium_queue_query: Query<Entity, With<TileLoadMedium>>,
    low_queue_query: Query<Entity, With<TileLoadLow>>,
    render_queue_query: Query<Entity, With<TileToRender>>,
) {
}
