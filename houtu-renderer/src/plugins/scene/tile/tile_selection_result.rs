#[derive(Debug, PartialEq)]
pub enum TileSelectionResult {
    NONE = 0,
    CULLED = 1,
    RENDERED = 2,
    REFINED = 3,
    RENDERED_AND_KICKED = 2 | 4,
    REFINED_AND_KICKED = 3 | 4,
    CULLED_BUT_NEEDED = 1 | 8,
}
impl Default for TileSelectionResult {
    fn default() -> Self {
        Self::None
    }
}
impl TileSelectionResult {
    pub fn wasKicked(value: u32) {
        return value >= Self::RENDERED_AND_KICKED;
    }
    pub fn originalResult(value: u32) {
        return value & 3;
    }
    pub fn kick(value: u32) {
        return value | 4;
    }
}
