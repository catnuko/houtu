use std::borrow::{Borrow, BorrowMut};

use bevy::prelude::*;
use houtu_scene::{
    Cartesian3, GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache, EPSILON5,
};

use super::{
    height_map_terrain_data::HeightmapTerrainDataCom, quadtree::Quadtree,
    quadtree_tile::QuadtreeTile, tile_bounding_region::TileBoundingRegionCom,
};
use crate::{camera::GlobeCamera, quadtree::{indices_and_edges_cache::IndicesAndEdgesCacheArc, tile_key::TileKey}};

#[derive(Component)]
pub struct Load {
    pub load_priority: f64,
    pub queue_type: QueueType,
}

impl Load {
    pub fn new() -> Self {
        Self {
            load_priority: 0.,
            queue_type: QueueType::None,
        }
    }
}
#[derive(PartialEq, Eq)]
pub enum QueueType {
    None,
    High,
    Medium,
    Low,
}
impl Default for QueueType {
    fn default() -> Self {
        QueueType::None
    }
}
#[derive(Resource)]
pub struct Queue {
    pub tile_load_queue_high: Vec<Entity>,
    pub tile_load_queue_medium: Vec<Entity>,
    pub tile_load_queue_low: Vec<Entity>,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            tile_load_queue_high: Vec::new(),
            tile_load_queue_medium: Vec::new(),
            tile_load_queue_low: Vec::new(),
        }
    }
}
impl Queue {
    pub fn clear(&mut self) {
        self.tile_load_queue_high.clear();
        self.tile_load_queue_medium.clear();
        self.tile_load_queue_low.clear();
    }
    pub fn enqueue(&mut self, queue_type: &QueueType, entity: Entity) {
        match queue_type {
            QueueType::High => self.tile_load_queue_high.push(entity),
            QueueType::Medium => self.tile_load_queue_medium.push(entity),
            QueueType::Low => self.tile_load_queue_low.push(entity),
            _ => {}
        }
    }
    pub fn size(&self) -> usize {
        let p2_count = self.tile_load_queue_high.len();
        let p3_count = self.tile_load_queue_medium.len();
        let p4_count = self.tile_load_queue_low.len();
        return p2_count + p3_count + p4_count;
    }
    pub fn empty(&self) -> bool {
        return self.size() == 0;
    }
}

pub fn clear_queue(mut queue: ResMut<Queue>) {
    queue.clear();
}

/// 修改Load就能实现入队
/// # Example
/// ```
/// load.load_priority = 0.5;
/// load.queue_type = QueueType::High;
/// ```
pub fn enqueue_update(
    mut globe_camera_query: Query<&mut GlobeCamera>,
    mut queue: ResMut<Queue>,
    mut tile_query: Query<(Entity, &mut Load, &TileBoundingRegionCom, &QuadtreeTile)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    for (entity, mut load, tile_bounding_region, quadtree_tile) in &mut tile_query {
        queue.enqueue(&load.queue_type, entity);
        load.queue_type = QueueType::None;
        let obb = tile_bounding_region.data.get_bounding_volume();
        if obb.is_none() {
            load.load_priority = 0.0;
            continue;
        }
        let obb = obb.unwrap();
        let camera_position = globe_camera.get_position_wc();
        let camera_direction = globe_camera.get_direction_wc();
        let mut tile_direction = obb.center - camera_position;
        let magnitude = tile_direction.magnitude();
        if magnitude < EPSILON5 {
            load.load_priority = 0.0;
            continue;
        }
        tile_direction = tile_direction / magnitude;
        load.load_priority = (1.0 - tile_direction.dot(camera_direction)) * quadtree_tile.distance;
    }
}
pub fn load_tile(
    mut commands: Commands,
    queue: Res<Queue>,
    mut quadtree_tile_query: Query<(&TileKey, &mut QuadtreeTile), Without<HeightmapTerrainDataCom>>,
    quadtree: Res<Quadtree>,
    mut indices_and_edges_cache: ResMut<IndicesAndEdgesCacheArc>,
) {
    load_tile_single_queue(
        &mut commands,
        &queue,
        QueueType::High,
        &mut quadtree_tile_query,
        &quadtree,
        &mut indices_and_edges_cache,
    );
    load_tile_single_queue(
        &mut commands,
        &queue,
        QueueType::Medium,
        &mut quadtree_tile_query,
        &quadtree,
        &mut indices_and_edges_cache,
    );
    load_tile_single_queue(
        &mut commands,
        &queue,
        QueueType::Low,
        &mut quadtree_tile_query,
        &quadtree,
        &mut indices_and_edges_cache,
    );
}
pub fn load_tile_single_queue(
    commands: &mut Commands,
    queue: &Queue,
    queue_type: QueueType,
    quadtree_tile_query: &mut Query<
        (&TileKey, &mut QuadtreeTile),
        Without<HeightmapTerrainDataCom>,
    >,
    quadtree: &Quadtree,
    indices_and_edges_cache_arc: &mut IndicesAndEdgesCacheArc,
) {
    // let load_queue = match queue_type {
    //     QueueType::High => &queue.tile_load_queue_high,
    //     QueueType::Medium => &queue.tile_load_queue_medium,
    //     QueueType::Low => &queue.tile_load_queue_low,
    //     QueueType::None => {
    //         panic!("error");
    //     }
    // };
    // for entity in load_queue {
    //     if let Ok((tile_key, mut quadtree_tile)) = quadtree_tile_query.get_mut(*entity) {
    //         let mut terrain_data = quadtree.terrain_provider.request_tile_geometry().unwrap();
    //         terrain_data.createMesh::<GeographicTilingScheme>(
    //             &quadtree.tiling_scheme,
    //             tile_key.x,
    //             tile_key.y,
    //             tile_key.level,
    //             None,
    //             None,
    //             indices_and_edges_cache_arc.get_cloned_cache(),
    //         );
    //         commands
    //             .entity(*entity)
    //             .insert(HeightmapTerrainDataCom(terrain_data));
    //         quadtree_tile.renderable = true;
    //     }
    // }
}
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Queue::default())
            .add_systems(PreUpdate, (clear_queue))
            .add_systems(
                PostUpdate,
                (enqueue_update, load_tile.after(enqueue_update)),
            );
    }
}
