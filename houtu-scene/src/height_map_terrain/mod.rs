// mod create_mesh_job;
pub struct HeightmapTerrainData {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
pub struct HeightmapTerrainDataStructure {
    pub heightScale: f32,
    pub heightOffset: f32,
    pub elementsPerHeight: f32,
    pub stride: f32,
    pub elementMultiplier: f32,
    pub isBigEndian: f32,
    pub lowestEncodedHeight: f32,
    pub highestEncodedHeight: f32,
}
