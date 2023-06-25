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

pub struct CreateTileJob {
    pub terrain_data: Arc<Mutex<HeightmapTerrainData>>,
    pub indicesAndEdgesCache: Arc<Mutex<IndicesAndEdgesCache>>,
    pub tiling_scheme: GeographicTilingScheme,
    pub key: TileKey,
    pub entity: Entity,
}
pub struct CreateTileJobOutcome {
    pub entity: Entity,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}
impl Job for CreateTileJob {
    type Outcome = Result<CreateTileJobOutcome, Error>;
    fn name(&self) -> String {
        format!("create vertice ",)
    }
    fn perform(self, context: Context) -> AsyncReturn<Self::Outcome> {
        Box::pin(async move {
            let fetch = async {
                self.terrain_data
                    .lock()
                    .expect("terrain_data.lock")
                    .createMesh(
                        &self.tiling_scheme,
                        self.key.x,
                        self.key.y,
                        self.key.level,
                        None,
                        None,
                        self.indicesAndEdgesCache,
                    )
                    .await;
                Ok(CreateTileJobOutcome {
                    entity: self.entity,
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
