use bevy::{
    asset::VisitAssetDependencies,
    ecs::system::lifetimeless::SRes,
    prelude::Asset,
    reflect::TypePath,
    render::{render_asset::RenderAsset, renderer::RenderDevice},
};
use houtu_scene::HeightmapTerrainData;

use crate::quantized_mesh_terrain_data::QuantizedMeshTerrainData;

#[derive(Asset, TypePath)]
pub enum TerrainDataAsset {
    HeightmapTerrainData(HeightmapTerrainData),
    QuantizedMeshTerrainData(QuantizedMeshTerrainData),
}
impl From<HeightmapTerrainData> for TerrainDataAsset {
    fn from(value: HeightmapTerrainData) -> Self {
        Self::HeightmapTerrainData(value)
    }
}
impl From<QuantizedMeshTerrainData> for TerrainDataAsset {
    fn from(value: QuantizedMeshTerrainData) -> Self {
        Self::QuantizedMeshTerrainData(value)
    }
}
// pub struct GpuTerrainDataAsset {}
// impl RenderAsset for TerrainDataAsset {
//     type ExtractedAsset = TerrainDataAsset;

//     type PreparedAsset = GpuTerrainDataAsset;

//     type Param = SRes<RenderDevice>;
//     fn extract_asset(&self) -> Self::ExtractedAsset {
//         self.clone()
//     }
//     fn prepare_asset(
//         extracted_asset: Self::ExtractedAsset,
//         param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
//     ) -> Result<
//         Self::PreparedAsset,
//         bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
//     > {

//     }
// }
