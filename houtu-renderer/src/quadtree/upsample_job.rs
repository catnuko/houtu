use bevy::prelude::*;
use houtu_jobs::{AsyncReturn, Context, Job};
use houtu_scene::{GeographicTilingScheme, HeightmapTerrainData};
use std::{
    io,
    sync::{Arc, Mutex},
};

use super::tile_key::TileKey;

pub struct UpsampleJob {
    pub terrain_data: Arc<Mutex<HeightmapTerrainData>>,
    pub tiling_scheme: GeographicTilingScheme,
    pub key: TileKey,
    pub parent_key: TileKey,
}
pub struct UpsampleJobOutcome {
    pub terrain_data: Option<HeightmapTerrainData>,
    pub key: TileKey,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}
impl Job for UpsampleJob {
    type Outcome = Result<UpsampleJobOutcome, Error>;
    fn name(&self) -> String {
        format!("create vertice ",)
    }
    fn perform(self, _context: Context) -> AsyncReturn<Self::Outcome> {
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
                    );
                Ok(UpsampleJobOutcome {
                    terrain_data: new_terrain_data,
                    key: self.key,
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
