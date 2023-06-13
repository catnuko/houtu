#[derive(Clone, PartialEq, Debug)]
pub struct NodeId {
    pub x: u32,
    pub y: u32,
    pub level: u32,
}
impl NodeId {
    pub fn southwest(&self) -> NodeId {
        NodeId {
            x: self.x * 2,
            y: self.y * 2 + 1,
            level: self.level + 1,
        }
    }
    pub fn southeast(&self) -> NodeId {
        NodeId {
            x: self.x * 2 + 1,
            y: self.y * 2 + 1,
            level: self.level + 1,
        }
    }
    pub fn northwest(&self) -> NodeId {
        NodeId {
            x: self.x * 2,
            y: self.y * 2,
            level: self.level + 1,
        }
    }
    pub fn northeast(&self) -> NodeId {
        NodeId {
            x: self.x * 2 + 1,
            y: self.y * 2,
            level: self.level + 1,
        }
    }
}
