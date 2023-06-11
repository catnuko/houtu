//! Contains LodVec trait, which is needed for the coordinate system to be used in a tree.
//! Sample implementations for this are in coords.rs.

/// trait for defining a Level of Detail vector.
/// such a vector contains the current position in the octree (3d coords), as well as the lod level it's at, in integer coords.
pub trait LodVec: std::hash::Hash + Eq + Sized + Copy + Clone + Send + Sync + Default {
    /// gets one of the child node position of this node, defined by it's index.
    fn get_child(self, index: usize) -> Self;

    /// get the number of child nodes a node can have in the tree.
    fn num_children() -> usize;

    /// returns the lod vector as if it's at the root of the tree.
    fn root() -> Self;

    /// wether the node can subdivide, compared to another node and the required detail.
    ///
    /// Assumes self is the target position for a lod.
    ///
    /// The depth determines the max lod level allowed, detail determines the amount of chunks around the target.
    ///
    /// if the detail is 0, this may only return true if self is inside the node.
    ///
    /// The implementation used in the QuadVec implementation is as follows:
    /// ```rust
    /// # struct Chunk { x: u64, y: u64, depth: u8 }
    /// # impl Chunk {
    /// fn can_subdivide(self, node: Self, detail: u64) -> bool {
    ///    // return early if the level of this chunk is too high
    ///    if node.depth >= self.depth {
    ///        return false;
    ///    }
    ///
    ///    // difference in lod level between the target and the node
    ///    let level_difference = self.depth - node.depth;
    ///
    ///    // minimum corner of the bounding box
    ///    let min = (
    ///        (node.x << (level_difference + 1))
    ///            .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
    ///        (node.y << (level_difference + 1))
    ///            .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
    ///    );
    ///
    ///    // max as well
    ///    let max = (
    ///        (node.x << (level_difference + 1))
    ///            .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
    ///        (node.y << (level_difference + 1))
    ///            .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
    ///    );
    ///
    ///    // local position of the target, which is one lod level higher to allow more detail
    ///    let local = (self.x << 1, self.y << 1);
    ///
    ///    // check if the target is inside of the bounding box
    ///    local.0 >= min.0 && local.0 < max.0 && local.1 >= min.1 && local.1 < max.1
    /// }
    /// # }
    /// ```
    fn can_subdivide(self, node: Self, detail: u64) -> bool;

    /// check if this chunk is inside of a bounding box
    /// where min is the lowest corner of the box, and max is the highest corner
    /// The implementation for QuadVec is as follows:
    /// ```rust
    /// # struct Chunk { x: u64, y: u64, depth: u8 }
    /// # impl Chunk {
    /// // get the lowest lod level
    /// let level = self.depth.min(min.depth.min(max.depth));
    ///
    /// // bring all coords to the lowest level
    /// let self_difference = self.depth - level;
    /// let min_difference = min.depth - level;
    /// let max_difference = max.depth - level;
    ///
    //// // get the coords to that level
    /// let self_x = self.x >> self_difference;
    /// let self_y = self.y >> self_difference;
    ///
    /// let min_x = min.x >> min_difference;
    /// let min_y = min.y >> min_difference;
    ///
    /// let max_x = max.x >> max_difference;
    /// let max_y = max.y >> max_difference;
    ///
    /// // then check if we are inside the AABB
    /// self.depth as u64 <= max_depth
    /// 	&& self_x >= min_x
    /// 	&& self_x < max_x
    /// 	&& self_y >= min_y
    /// 	&& self_y < max_y
    /// # }
    /// ```
    fn is_inside_bounds(self, min: Self, max: Self, max_depth: u64) -> bool;

    /// Wether this node contains a child node
    fn contains_child_node(self, child: Self) -> bool;
}
