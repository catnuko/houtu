#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
}
