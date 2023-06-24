use super::tile_node::TileNode;
use super::Quadrant;
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NodeChildren {
    pub(super) northwest: TileNode,
    pub(super) northeast: TileNode,
    pub(super) southwest: TileNode,
    pub(super) southeast: TileNode,
}

impl Default for NodeChildren {
    fn default() -> Self {
        Self {
            northwest: TileNode::None,
            northeast: TileNode::None,
            southwest: TileNode::None,
            southeast: TileNode::None,
        }
    }
}

impl Index<Quadrant> for NodeChildren {
    type Output = TileNode;

    fn index(&self, quadrant: Quadrant) -> &TileNode {
        match quadrant {
            Quadrant::Northwest => &self.northwest,
            Quadrant::Northeast => &self.northeast,
            Quadrant::Southwest => &self.southwest,
            Quadrant::Southeast => &self.southeast,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<Quadrant> for NodeChildren {
    fn index_mut(&mut self, quadrant: Quadrant) -> &mut TileNode {
        match quadrant {
            Quadrant::Northwest => &mut self.northwest,
            Quadrant::Northeast => &mut self.northeast,
            Quadrant::Southwest => &mut self.southwest,
            Quadrant::Southeast => &mut self.southeast,
            _ => unreachable!(),
        }
    }
}

impl IntoIterator for NodeChildren {
    type Item = TileNode;
    type IntoIter = ::std::vec::IntoIter<TileNode>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.northwest,
            self.northeast,
            self.southwest,
            self.southeast,
        ]
        .into_iter()
    }
}
