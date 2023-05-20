use bevy::prelude::*;
use houtu_scene::{
    GeographicProjection, GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache,
    TilingScheme,
};

pub struct ScenePlugin;

impl bevy::app::Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiling_scheme = GeographicTilingScheme::default();
    let min_longitude = -180.0;
    let max_longitude = 180.0;
    let min_latitude = -90.0;
    let max_latitude = 90.0;
    let level = 3;
    let num_of_x_tiles = tiling_scheme.get_number_of_x_tiles_at_level(level);
    let num_of_y_tiles = tiling_scheme.get_number_of_y_tiles_at_level(level);
    let mut indicesAndEdgesCache = IndicesAndEdgesCache::new();
    for y in 0..num_of_y_tiles {
        for x in 0..num_of_x_tiles {
            let width: u32 = 16;
            let height: u32 = 16;
            let buffer: Vec<f64> = vec![0.; (width * height) as usize];
            let mut heigmapTerrainData = HeightmapTerrainData::new(
                buffer, width, height, None, None, None, None, None, None, None,
            );
            heigmapTerrainData._createMeshSync(
                &tiling_scheme,
                x,
                y,
                level,
                None,
                None,
                &mut indicesAndEdgesCache,
            )
        }
    }
}
