#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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
    pub fn wasKicked(value: &TileSelectionResult) -> bool {
        return *value >= Self::RENDERED_AND_KICKED;
    }
    pub fn originalResult(value: &TileSelectionResult) -> u8 {
        let v = value.clone();
        return (v as u8) & 3;
    }
    pub fn kick(value: &TileSelectionResult) -> u8 {
        let v = value.clone();
        return (v as u8) | 4;
    }
    pub fn from_u8(v: u8) -> Self {
        if v == 0 {
            return Self::NONE;
        } else if v == 1 {
            return Self::CULLED;
        } else if v == 2 {
            return Self::RENDERED;
        } else if v == 3 {
            return Self::REFINED;
        } else if v == 2 | 4 {
            return Self::RENDERED_AND_KICKED;
        } else if v == 3 | 4 {
            return Self::REFINED_AND_KICKED;
        } else if v == 1 | 8 {
            return Self::CULLED_BUT_NEEDED;
        } else {
            return Self::NONE;
        }
    }
}
