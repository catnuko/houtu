use super::{
    create_terrain_mesh_job::CreateTileJob,
    globe_surface_tile::GlobeSurfaceTile,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_tile::{QuadtreeTileLoadState, QuadtreeTileParent, TileNode},
    upsample_job::UpsampleJob,
    TileKey,
};
use bevy::prelude::*;
use houtu_jobs::{FinishedJobs, JobSpawner};
use houtu_scene::{
    Ellipsoid, GeographicTilingScheme, HeightmapTerrainData, Rectangle, TerrainMesh, TilingScheme,
};
use std::{
    f32::consts::E,
    f64::consts::{E as Ef64, PI},
    sync::{Arc, Mutex},
};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainDataSource::new());
        app.add_system(update_system);
    }
}
#[derive(PartialEq, Eq)]
pub enum TerrainDataSourceState {
    FAILED = 0,
    UNLOADED = 1,
    RECEIVING = 2,
    RECEIVED = 3,
    TRANSFORMING = 4,
    TRANSFORMED = 5,
    READY = 6,
}
#[derive(Component)]
pub struct TerrainDataSourceData {
    pub state: TerrainDataSourceState,
    terrain_data: Option<Arc<Mutex<HeightmapTerrainData>>>,
}
impl Default for TerrainDataSourceData {
    fn default() -> Self {
        Self {
            state: TerrainDataSourceState::UNLOADED,
            terrain_data: None,
        }
    }
}
impl TerrainDataSourceData {
    pub fn get_cloned_terrain_data(&self) -> Arc<Mutex<HeightmapTerrainData>> {
        self.terrain_data.as_ref().unwrap().clone()
    }
    pub fn has_mesh(&self) -> bool {
        if let Some(v) = self.terrain_data.as_ref() {
            v.clone().lock().unwrap().has_mesh()
        } else {
            false
        }
    }
    pub fn has_terrain_data(&self) -> bool {
        return self.terrain_data.is_some();
    }
    pub fn was_created_by_upsampling(&self) -> bool {
        return self
            .terrain_data
            .as_ref()
            .expect("globe_surface_tile.terrainData")
            .lock()
            .expect("globe_surface_tile.terrainData.lock")
            .was_created_by_upsampling();
    }
    pub fn is_child_available(&self, parent_key: &TileKey, key: &TileKey) -> bool {
        self.terrain_data
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .is_child_available(parent_key.x, parent_key.y, key.x, key.y)
    }
    pub fn can_upsample(&self) -> bool {
        return self
            .terrain_data
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .can_upsample();
    }
    pub fn get_mesh(&self) -> Option<Mesh> {
        if let Some(terrain_data) = self.terrain_data.as_ref() {
            if let Some(v) = terrain_data.clone().lock().unwrap()._mesh.as_ref() {
                let mesh: Mesh = v.into();
                return Some(mesh);
            }
        }
        return None;
    }
    // pub fn get_terrain_mesh(&self) -> Option<&TerrainMesh> {
    //     if let Some(terrain_data) = self.terrain_data.as_ref() {
    //         if let Some(v) = terrain_data.clone().lock().unwrap()._mesh.as_ref() {
    //             return Some(v);
    //         }
    //     }
    //     return None;
    // }
    pub fn set_terrain_data(&mut self, new_terrain_data: HeightmapTerrainData) {
        self.terrain_data = Some(Arc::new(Mutex::new(new_terrain_data)));
        self.state = TerrainDataSourceState::RECEIVED;
    }
}
fn update_system(
    mut terrain_datasource: ResMut<TerrainDataSource>,
    indices_and_edges_cache: Res<IndicesAndEdgesCacheArc>,
    mut job_spawner: JobSpawner,
    mut finished_jobs: FinishedJobs,
    mut query: Query<(
        Entity,
        &TileKey,
        &mut TerrainDataSourceData,
        &QuadtreeTileParent,
        &mut QuadtreeTileLoadState,
    )>,
) {
    let mut do_upsample_list: Vec<(Entity, Entity, TileKey)> = vec![];
    for (entity, key, mut data, parent, mut state) in &mut query {
        if data.state == TerrainDataSourceState::FAILED {
            match parent.0 {
                TileNode::None => {
                    *state = QuadtreeTileLoadState::FAILED;
                }
                TileNode::Internal(v) => {
                    do_upsample_list.push((entity, v, key.clone()));
                }
            }
        }
        if data.state == TerrainDataSourceState::UNLOADED {
            data.state = TerrainDataSourceState::RECEIVING;
            let value = terrain_datasource
                .request_tile_geometry()
                .expect("terrain_datasource.request_tile_geometry");
            data.set_terrain_data(value);
        }
        if data.state == TerrainDataSourceState::RECEIVING {}
        if data.state == TerrainDataSourceState::RECEIVED {
            job_spawner.spawn(CreateTileJob {
                terrain_data: data.get_cloned_terrain_data(),
                key: key.clone(),
                tiling_scheme: terrain_datasource.tiling_scheme.clone(),
                indices_and_edges_cache: indices_and_edges_cache.get_cloned_cache(),
                entity: entity,
            });
            data.state = TerrainDataSourceState::TRANSFORMING;
        }
        if data.state == TerrainDataSourceState::TRANSFORMING {}
        if data.state == TerrainDataSourceState::TRANSFORMED {
            data.state = TerrainDataSourceState::READY;
        }
        if data.state == TerrainDataSourceState::READY {}
    }
    while let Some(result) = finished_jobs.take_next::<CreateTileJob>() {
        if let Ok(res) = result {
            let mut data = query
                .get_component_mut::<TerrainDataSourceData>(res.entity)
                .unwrap();
            data.state = TerrainDataSourceState::TRANSFORMED;
        }
    }
    do_upsample_list
        .iter()
        .for_each(|(entity, parent_entity, key)| {
            let (terrain_data, parent_key) = {
                // let mut world = params_set.p1();
                let data = query
                    .get_component::<TerrainDataSourceData>(*parent_entity)
                    .unwrap();
                if data.terrain_data.is_none() {
                    return;
                }
                let parent_key = query.get_component::<TileKey>(*parent_entity).unwrap();
                (data.get_cloned_terrain_data(), parent_key.clone())
            };

            job_spawner.spawn(UpsampleJob {
                terrain_data: terrain_data,
                tiling_scheme: terrain_datasource.tiling_scheme.clone(),
                parent_key: parent_key,
                key: key.clone(),
                entity: *entity,
            });
        });
    while let Some(result) = finished_jobs.take_next::<UpsampleJob>() {
        if let Ok(res) = result {
            let mut globe_surface_tile = query
                .get_component_mut::<TerrainDataSourceData>(res.entity)
                .unwrap();
            if let Some(new_terrain_data) = res.terrain_data {
                globe_surface_tile.set_terrain_data(new_terrain_data);
            } else {
                globe_surface_tile.state = TerrainDataSourceState::FAILED;
            }
        }
    }
}

#[derive(Resource)]
pub struct TerrainDataSource {
    pub tiling_scheme: GeographicTilingScheme,
    _level_zero_maximum_geometric_error: f64,
    pub ready: bool,
    pub rectangle: Rectangle,
}
impl TerrainDataSource {
    pub fn new() -> Self {
        let tiling_scheme = GeographicTilingScheme::default();
        let _level_zero_maximum_geometric_error =
            get_level_zero_maximum_geometric_error(&tiling_scheme);

        Self {
            tiling_scheme: tiling_scheme,
            _level_zero_maximum_geometric_error: _level_zero_maximum_geometric_error,
            ready: true,
            rectangle: Rectangle::MAX_VALUE.clone(),
        }
    }
    pub fn get_tile_data_available(&self, key: &TileKey) -> Option<bool> {
        return None;
    }
    pub fn load_tile_data_availability(&self, key: &TileKey) -> Option<bool> {
        return None;
    }
    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        return self._level_zero_maximum_geometric_error / (1 << level) as f64;
    }
    pub fn can_refine(&self, globe_surface_tile: &TerrainDataSourceData, key: &TileKey) -> bool {
        if globe_surface_tile.terrain_data.is_some() {
            return true;
        }
        let new_key = TileKey::new(key.x * 2, key.y * 2, key.level + 1);
        let child_available = self.get_tile_data_available(&new_key);
        return child_available != None;
    }

    pub fn request_tile_geometry(&self) -> Option<HeightmapTerrainData> {
        let width = 16;
        let height = 16;
        return Some(HeightmapTerrainData::new(
            vec![0.; width * height],
            width as u32,
            height as u32,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }
}
fn get_level_zero_maximum_geometric_error(tiling_scheme: &GeographicTilingScheme) -> f64 {
    return get_estimated_level_zero_geometric_error_for_a_heightmap(
        &tiling_scheme.ellipsoid,
        64,
        tiling_scheme.get_number_of_tiles_at_level(0),
    );
}
fn get_estimated_level_zero_geometric_error_for_a_heightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    number_of_tiles_at_level_zero: u32,
) -> f64 {
    return ((ellipsoid.maximum_radius * 2. * PI * 0.25)
        / (tile_image_width as f64 * number_of_tiles_at_level_zero as f64));
}