use bevy::math::UVec3;

#[derive(Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub struct TileKey {
    pub x: u32,
    pub y: u32,
    pub level: u32,
}
impl TileKey {
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        Self { x, y, level }
    }

    pub fn get_id(&self) -> String {
        format!("{}_{}_{}", self.x, self.y, self.level)
    }
    pub fn southwest(&self) -> TileKey {
        TileKey {
            x: self.x * 2,
            y: self.y * 2 + 1,
            level: self.level + 1,
        }
    }
    pub fn southeast(&self) -> TileKey {
        TileKey {
            x: self.x * 2 + 1,
            y: self.y * 2 + 1,
            level: self.level + 1,
        }
    }
    pub fn northwest(&self) -> TileKey {
        TileKey {
            x: self.x * 2,
            y: self.y * 2,
            level: self.level + 1,
        }
    }
    pub fn northeast(&self) -> TileKey {
        TileKey {
            x: self.x * 2 + 1,
            y: self.y * 2,
            level: self.level + 1,
        }
    }
    pub fn parent(&self) -> Option<TileKey> {
        if self.level != 0 {
            let parent_x = (self.x / 2) | 0;
            let parent_y = (self.y / 2) | 0;
            let parent_level = self.level - 1;
            Some(TileKey {
                x: parent_x,
                y: parent_y,
                level: parent_level,
            })
        } else {
            None
        }
    }
}

impl From<TileKey> for UVec3 {
    fn from(pos: TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<&TileKey> for UVec3 {
    fn from(pos: &TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<UVec3> for TileKey {
    fn from(v: UVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            level: v.z,
        }
    }
}
