
use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    math::prelude::*,
    transform::components::Transform, reflect::DynamicMap,
};

use crate::layer_id::LayerId;
fn handle_load_file_events<F: geo_file_loader::FileLoader + Send + Sync + 'static>(
    mut load_event_reader: ResMut<Events<houtu_events::LoadFileEvent<F>>>,
    mut job_spawner: bevy_jobs::JobSpawner,
) where
    <F as geo_file_loader::FileLoader>::Error: Send + Sync + 'static,
{
    for event in load_event_reader.drain() {
        match event {
            houtu_events::LoadFileEvent::FromNetwork { url, crs, name } => {
                job_spawner.spawn(rgis_network::NetworkFetchJob { url, crs, name })
            }
            houtu_events::LoadFileEvent::FromBytes {
                file_name,
                file_loader,
                crs,
            } => job_spawner.spawn(crate::jobs::LoadFileJob {
                file_loader,
                source_crs: crs,
                name: file_name,
            }),
        }
    }
}
