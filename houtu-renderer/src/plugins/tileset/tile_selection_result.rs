#[derive(Debug, PartialEq, Ord)]
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
        Self::NONE
    }
}
impl TileSelectionResult {
    pub fn wasKicked(value: TileSelectionResult) -> bool {
        return value >= Self::RENDERED_AND_KICKED;
    }
    pub fn originalResult(value: TileSelectionResult) -> u8 {
        return (value as u8) & 3;
    }
    pub fn kick(value: TileSelectionResult) -> u8 {
        return (value as u8) | 4;
    }
}
