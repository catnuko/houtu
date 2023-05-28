use bevy::prelude::*;

use crate::plugins::{camera::GlobeMapCamera, quadtree::QuadtreeTile};

use super::datasource::{DataSource, GlobeSurfaceTileDataSource};
#[derive(Debug, Resource)]
pub struct TileLayerLoader {
    pub datasourse: Box<GlobeSurfaceTileDataSource>,
    pub _tileLoadQueueHigh: Vec<QuadtreeTile>,
    pub _tileLoadQueueMedium: Vec<QuadtreeTile>,
    pub _tileLoadQueueLow: Vec<QuadtreeTile>,
    pub _tilesToRender: Vec<QuadtreeTile>,
}
impl TileLayerLoader {
    pub fn new(datasource: &GlobeSurfaceTileDataSource) -> Self {
        Self {
            datasourse: datasourse,
            _tileLoadQueueHigh: Vec::new(),
            _tileLoadQueueMedium: Vec::new(),
            _tileLoadQueueLow: Vec::new(),
            _tilesToRender: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum QueueType {
    High,
    Medium,
    Low,
    Render,
}

#[derive(Debug)]
pub struct EnQueue {
    pub tile: QuadtreeTile,
    pub queue_tyoe: QueueType,
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<EnQueue>();
        app.add_system(enqueue_and_dequeue);
    }
}
fn enqueue_and_dequeue(
    mut ev_levelup: EventReader<EnQueue>,
    mut tile_layer_laoder: ResMut<TileLayerLoader>,
    query: Query<&mut GlobeMapCamera>,
) {
    for globe_map_camera in query.iter() {
        for ev in ev_levelup.iter() {
            if !*(ev.tile).needsLoading {
                continue;
            }
            let loadPriority = tile_layer_laoder.datasource.computeTileLoadPriority(
                ev.tile,
                &globe_map_camera.position_cartesian,
                &globe_map_camera.direction,
            );
            *(ev.tile)._loadPriority = loadPriority;
            match ev.queue_tyoe {
                QueueType::High => tile_layer_laoder._tileLoadQueueHigh.push(ev.tile),
                QueueType::Medium => tile_layer_laoder._tileLoadQueueMedium.push(ev.tile),
                QueueType::Low => tile_layer_laoder._tileLoadQueueLow.push(ev.tile),
                QueueType::Render => tile_layer_laoder._tilesToRender.push(ev.tile),
            }
        }
    }
}
