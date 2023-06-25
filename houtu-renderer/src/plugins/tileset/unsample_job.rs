use bevy::{prelude::*, utils::Uuid};
use houtu_jobs::{
    AsyncReturn, Context, FinishedJobs, Job, JobOutcomePayload, JobSpawner, Progress,
};
use houtu_scene::{
    CreateVerticeReturn, GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache,
    Rectangle, TerrainMesh, TilingScheme, WebMercatorTilingScheme,
};
use std::{
    io,
    sync::{Arc, Mutex, RwLock},
};

use super::TileKey;

pub struct UnsampleJob {
    pub terrain_data: Arc<Mutex<HeightmapTerrainData>>,
    pub tiling_scheme: GeographicTilingScheme,
    pub key: TileKey,
    pub parent_key: TileKey,
    pub entity: Entity,
}
pub struct UnsampleJobOutcome {
    pub entity: Entity,
    pub terrain_data: Option<HeightmapTerrainData>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}
impl Job for UnsampleJob {
    type Outcome = Result<UnsampleJobOutcome, Error>;
    fn name(&self) -> String {
        format!("create vertice ",)
    }
    fn perform(self, context: Context) -> AsyncReturn<Self::Outcome> {
        Box::pin(async move {
            let fetch = async {
                let new_terrain_data = self
                    .terrain_data
                    .lock()
                    .expect("terrain_data.lock")
                    .upsample(
                        &self.tiling_scheme,
                        self.parent_key.x,
                        self.parent_key.y,
                        self.parent_key.level,
                        self.key.x,
                        self.key.y,
                        self.key.level,
                    )
                    .await;
                Ok(UnsampleJobOutcome {
                    entity: self.entity,
                    terrain_data: new_terrain_data,
                })
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(fetch)
            }
            #[cfg(target_arch = "wasm32")]
            {
                fetch.await
            }
        })
    }
    // fn spawn(self, commands: &mut bevy::ecs::system::Commands) {}
}
