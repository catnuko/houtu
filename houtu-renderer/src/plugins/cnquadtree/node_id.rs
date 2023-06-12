#[derive(Clone, PartialEq)]
pub struct NodeId {
    pub(super) x: u32,
    pub(super) y: u32,
    pub(super) level: u32,
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
