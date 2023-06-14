use bevy::prelude::*;
use houtu_scene::{Cartographic, EllipsoidalOccluder};

use super::tile_quad_node::{TileLoadHigh, TileLoadLow, TileLoadMedium};

#[derive(Resource, Clone, Debug)]
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
        }
    }
    /// 调用后将清空所有瓦片重新创建
    pub fn invalidateAllTiles(&mut self) {
        self._tilesInvalidated = true;
    }
    pub fn real_invalidateAllTiles(&mut self) {}
}
pub fn system(
    mut commands: Commands,
    mut tile_quad_tree: ResMut<TileQuadTree>,
    high_queue_query: Query<Entity, With<TileLoadHigh>>,
    medium_queue_query: Query<Entity, With<TileLoadMedium>>,
    low_queue_query: Query<Entity, With<TileLoadLow>>,
) {
    // 帧开始
    if (tile_quad_tree._tilesInvalidated) {
        tile_quad_tree.real_invalidateAllTiles();
        tile_quad_tree._tilesInvalidated = false;
    }
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
}
