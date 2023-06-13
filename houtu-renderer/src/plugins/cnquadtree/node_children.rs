use super::terrain_quadtree_node::TerrainQuadtreeNode;
use super::Quadrant;
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NodeChildren {
    pub(super) northwest: TerrainQuadtreeNode,
    pub(super) northeast: TerrainQuadtreeNode,
    pub(super) southwest: TerrainQuadtreeNode,
    pub(super) southeast: TerrainQuadtreeNode,
}

impl Default for NodeChildren {
    fn default() -> Self {
        Self {
            northwest: TerrainQuadtreeNode::None,
            northeast: TerrainQuadtreeNode::None,
            southwest: TerrainQuadtreeNode::None,
            southeast: TerrainQuadtreeNode::None,
        }
    }
}

impl Index<Quadrant> for NodeChildren {
    type Output = TerrainQuadtreeNode;

    fn index(&self, quadrant: Quadrant) -> &TerrainQuadtreeNode {
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
    fn index_mut(&mut self, quadrant: Quadrant) -> &mut TerrainQuadtreeNode {
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
    type Item = TerrainQuadtreeNode;
    type IntoIter = ::std::vec::IntoIter<TerrainQuadtreeNode>;

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
