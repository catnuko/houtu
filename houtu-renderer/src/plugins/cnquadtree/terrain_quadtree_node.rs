#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TerrainQuadtreeNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
}
