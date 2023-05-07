// mod create_mesh_job;
pub struct HeightmapTerrainData {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
pub struct HeightmapTerrainDataStructure {
    pub heightScale: f64,
    pub heightOffset: f64,
    pub elementsPerHeight: u32,
    pub stride: u32,
    pub elementMultiplier: f64,
    pub isBigEndian: bool,
    pub lowestEncodedHeight: f64,
    pub highestEncodedHeight: f64,
}
impl Default for HeightmapTerrainDataStructure {
    fn default() -> Self {
        HeightmapTerrainDataStructure {
            heightScale: 1.0,
            heightOffset: 0.0,
            elementsPerHeight: 1,
            stride: 1,
            elementMultiplier: 256.0,
            isBigEndian: false,
            lowestEncodedHeight: 0.0,
            highestEncodedHeight: 256.0,
        }
    }
}
