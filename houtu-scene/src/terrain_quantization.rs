#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TerrainQuantization {
    NONE = 0,
    BITS12 = 1,
}

impl Default for TerrainQuantization {
    fn default() -> Self {
        Self::NONE
    }
}
