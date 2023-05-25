use bevy::{prelude::*, utils::Uuid};
use houtu_jobs::{
    AsyncReturn, Context, FinishedJobs, Job, JobOutcomePayload, JobSpawner, Progress,
};
use houtu_scene::{
    CreateVerticeReturn, HeightmapTerrainData, IndicesAndEdgesCache, Rectangle, TerrainMesh,
    TilingScheme, WebMercatorTilingScheme,
};
use std::{
    io,
    sync::{Arc, RwLock},
};

use super::{tile::Tile, tile_layer::TileLayer, tile_state::TileState};
pub struct CreateTileJob {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub width: u32,
    pub height: u32,
}
pub struct CreateTileJobOutcome {
    result: CreateVerticeReturn,
    job: CreateTileJob,
    height_data: HeightmapTerrainData,
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
                let tiling_scheme = WebMercatorTilingScheme::default();

                let buffer: Vec<f64> = vec![0.; (self.width * self.height) as usize];
                let mut height_data = HeightmapTerrainData::new(
                    buffer,
                    self.width,
                    self.height,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                let mut indicesAndEdgesCache = IndicesAndEdgesCache::new();
                let result = height_data.create_vertice(
                    &tiling_scheme,
                    self.x,
                    self.y,
                    self.level,
                    None,
                    None,
                    &mut indicesAndEdgesCache,
                );
                Ok(CreateTileJobOutcome {
                    result: result,
                    job: self,
                    height_data: height_data,
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
pub fn handle_created_tile_system(
    mut finished_jobs: FinishedJobs,
    mut indicesAndEdgesCache: ResMut<IndicesAndEdgesCache>,
    mut tile_layer: ResMut<TileLayer>,
    // world: &mut World,
    mut commands: Commands,
) {
    while let Some(outcome) = finished_jobs.take_next::<CreateTileJob>() {
        match outcome {
            Ok(outcome) => {
                let x = outcome.job.x;
                let y = outcome.job.y;
                let level = outcome.job.level;
                let terrain_mesh = create_terrain_mesh(outcome, &mut indicesAndEdgesCache);

                if let Some(entity) = tile_layer.get_tile_entity(x, y, level) {
                    let cloned_entity = entity.clone();
                    commands.add(move |world: &mut World| {
                        if let Some(mut tile) = world.get_mut::<Tile>(cloned_entity) {
                            tile.terrain_mesh = Some(terrain_mesh);
                            tile.state = TileState::READY
                        } else {
                            bevy::log::error!("瓦片实体中找不到瓦片组件,{},{},{}", x, y, level)
                        }
                    })
                } else {
                    bevy::log::error!("创建的瓦片后发现瓦片在图层中不存在,{},{},{}", x, y, level)
                }
            }
            Err(e) => {
                bevy::log::error!("Encountered error when loading file: {:?}", e);
            }
        }
    }
}
pub fn create_terrain_mesh(
    ouotcome: CreateTileJobOutcome,
    indicesAndEdgesCache: &mut ResMut<IndicesAndEdgesCache>,
) -> TerrainMesh {
    let indicesAndEdges;
    if (ouotcome.height_data._skirtHeight.unwrap() > 0.0) {
        indicesAndEdges = indicesAndEdgesCache
            .getRegularGridAndSkirtIndicesAndEdgeIndices(ouotcome.job.width, ouotcome.job.height);
    } else {
        indicesAndEdges = indicesAndEdgesCache
            .getRegularGridIndicesAndEdgeIndices(ouotcome.job.width, ouotcome.job.height);
    }

    let vertexCountWithoutSkirts = 0;
    return TerrainMesh::new(
        ouotcome.result.relativeToCenter.unwrap(),
        ouotcome.result.vertices,
        indicesAndEdges.indices,
        indicesAndEdges.indexCountWithoutSkirts,
        vertexCountWithoutSkirts,
        ouotcome.result.minimumHeight,
        ouotcome.result.maximumHeight,
        ouotcome.result.boundingSphere3D,
        ouotcome.result.occludeePointInScaledSpace,
        ouotcome.result.encoding.stride,
        ouotcome.result.orientedBoundingBox,
        ouotcome.result.encoding,
        indicesAndEdges.westIndicesSouthToNorth,
        indicesAndEdges.southIndicesEastToWest,
        indicesAndEdges.eastIndicesNorthToSouth,
        indicesAndEdges.northIndicesWestToEast,
    );
}
