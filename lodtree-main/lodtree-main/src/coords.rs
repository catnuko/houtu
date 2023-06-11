//! Contains coordinate structs, QuadVec for quadtrees, and OctVec for octrees, as well as their LodVec implementation

use crate::traits::LodVec;

/// A Lod Vector for use in a quadtree.
/// It subdivides into 4 children of equal size.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
pub struct QuadVec {
    /// x position in the quadtree.
    pub x: u64,

    /// y position in the quadtree.
    pub y: u64,

    /// lod depth in the quadtree.
    /// this is limited, hence we use u8.
    pub depth: u8,
}

impl QuadVec {
    /// creates a new vector from the raw x and y coords.
    /// # Args
    /// * `x` The x position in the tree. Allowed range scales with the depth (doubles as the depth increases by one)
    /// * `y` The x position in the tree. Allowed range scales with the depth (doubles as the depth increases by one)
    /// * `depth` the lod depth the coord is at. This is soft limited at roughly 60, and the tree might behave weird if it gets higher
    #[inline]
    pub fn new(x: u64, y: u64, depth: u8) -> Self {
        Self { x, y, depth }
    }

    /// creates a new vector from floating point coords
    /// mapped so that (0, 0) is the bottom left corner and (1, 1) is the top right
    /// # Args
    /// * `x` x coord of the float vector, from 0 to 1
    /// * `y` y coord of the float vector, from 0 to 1
    /// * `depth` The lod depth of the coord
    #[inline]
    pub fn from_float_coords(x: f64, y: f64, depth: u8) -> Self {
        // scaling factor due to the lod depth
        let scale_factor = (1 << depth) as f64;

        // and get the actual coord
        Self {
            x: (x * scale_factor) as u64,
            y: (y * scale_factor) as u64,
            depth,
        }
    }

    /// converts the coord into float coords.
    /// Returns a tuple of (x: f64, y: f64) to represent the coordinates, this is the lower left corner.
    #[inline]
    pub fn get_float_coords(self) -> (f64, f64) {
        // scaling factor to scale the coords down with
        let scale_factor = 1.0 / (1 << self.depth) as f64;

        // and the x and y coords
        (self.x as f64 * scale_factor, self.y as f64 * scale_factor)
    }

    /// gets the size the chunk of this lod vector takes up, with the root taking up.
    #[inline]
    pub fn get_size(self) -> f64 {
        1.0 / (1 << self.depth) as f64
    }
}

impl LodVec for QuadVec {
    #[inline]
    fn num_children() -> usize {
        4
    }

    #[inline]
    fn root() -> Self {
        Self {
            x: 0,
            y: 0,
            depth: 0,
        }
    }

    #[inline]
    fn get_child(self, index: usize) -> Self {
        // the positions, doubled in scale
        let x = self.x << 1;
        let y = self.y << 1;

        // and how much to increment them with
        let increment_x = index as u64 & 1;
        let increment_y = (index as u64 & 2) >> 1;

        // and return
        Self {
            x: x + increment_x,
            y: y + increment_y,
            depth: self.depth + 1,
        }
    }

    #[inline]
    fn can_subdivide(self, node: Self, detail: u64) -> bool {
        // return early if the level of this chunk is too high
        if node.depth >= self.depth {
            return false;
        }

        // difference in lod level between the target and the node
        let level_difference = self.depth - node.depth;

        // minimum corner of the bounding box
        let min = (
            (node.x << (level_difference + 1))
                .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
            (node.y << (level_difference + 1))
                .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
        );

        // max as well
        let max = (
            (node.x << (level_difference + 1))
                .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
            (node.y << (level_difference + 1))
                .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
        );

        // local position of the target, which is one lod level higher to allow more detail
        let local = (self.x << 1, self.y << 1);

        // check if the target is inside of the bounding box
        local.0 >= min.0 && local.0 < max.0 && local.1 >= min.1 && local.1 < max.1
    }

    fn is_inside_bounds(self, min: Self, max: Self, max_depth: u64) -> bool {
        // get the lowest lod level
        let level = self.depth.min(min.depth.min(max.depth));

        // bring all coords to the lowest level
        let self_difference = self.depth - level;
        let min_difference = min.depth - level;
        let max_difference = max.depth - level;

        // get the coords to that level
        let self_x = self.x >> self_difference;
        let self_y = self.y >> self_difference;

        let min_x = min.x >> min_difference;
        let min_y = min.y >> min_difference;

        let max_x = max.x >> max_difference;
        let max_y = max.y >> max_difference;

        // then check if we are inside the AABB
        self.depth as u64 <= max_depth
            && self_x >= min_x
            && self_x < max_x
            && self_y >= min_y
            && self_y < max_y
    }

    #[inline]
    fn contains_child_node(self, child: Self) -> bool {
        // basically, move the child node up to this level and check if they're equal
        let level_difference = child.depth - self.depth;

        // and move
        let x = child.x >> level_difference;
        let y = child.y >> level_difference;

        // and check
        self.x == x && self.y == y
    }
}

/// A Lod Vector for use in an octree.
/// It subdivides into 8 children of equal size.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
pub struct OctVec {
    /// x position in the octree.
    pub x: u64,

    /// y position in the octree.
    pub y: u64,

    /// z position in the octree.
    pub z: u64,

    /// lod depth in the octree.
    /// this is limited, hence we use u8.
    pub depth: u8,
}

impl OctVec {
    /// creates a new vector from the raw x and y coords.
    /// # Args
    /// * `x` The x position in the tree. Allowed range scales with the depth (doubles as the depth increases by one)
    /// * `y` The y position in the tree. Allowed range scales with the depth (doubles as the depth increases by one)
    /// * `z` The z position in the tree. Allowed range scales with the depth (doubles as the depth increases by one)
    /// * `depth` the lod depth the coord is at. This is soft limited at roughly 60, and the tree might behave weird if it gets higher.
    #[inline]
    pub fn new(x: u64, y: u64, z: u64, depth: u8) -> Self {
        Self { x, y, z, depth }
    }

    /// creates a new vector from floating point coords.
    /// mapped so that (0, 0, 0) is the front bottom left corner and (1, 1, 1) is the back top right.
    /// # Args
    /// * `x` x coord of the float vector, from 0 to 1
    /// * `y` y coord of the float vector, from 0 to 1
    /// * `z` z coord of the float vector, from 0 to 1
    /// * `depth` The lod depth of the coord
    #[inline]
    pub fn from_float_coords(x: f64, y: f64, z: f64, depth: u8) -> Self {
        // scaling factor due to the lod depth
        let scale_factor = (1 << depth) as f64;

        // and get the actual coord
        Self {
            x: (x * scale_factor) as u64,
            y: (y * scale_factor) as u64,
            z: (z * scale_factor) as u64,
            depth,
        }
    }

    /// converts the coord into float coords.
    /// Returns a tuple of (x: f64, y: f64, z: f64) to represent the coordinates, at the front bottom left corner.
    #[inline]
    pub fn get_float_coords(self) -> (f64, f64, f64) {
        // scaling factor to scale the coords down with
        let scale_factor = 1.0 / (1 << self.depth) as f64;

        // and the x and y coords
        (
            self.x as f64 * scale_factor,
            self.y as f64 * scale_factor,
            self.z as f64 * scale_factor,
        )
    }

    /// gets the size the chunk of this lod vector takes up, with the root taking up.
    #[inline]
    pub fn get_size(self) -> f64 {
        1.0 / (1 << self.depth) as f64
    }
}

impl LodVec for OctVec {
    #[inline]
    fn num_children() -> usize {
        8
    }

    #[inline]
    fn root() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            depth: 0,
        }
    }

    #[inline]
    fn get_child(self, index: usize) -> Self {
        // the positions, doubled in scale
        let x = self.x << 1;
        let y = self.y << 1;
        let z = self.z << 1;

        // and how much to increment them with
        let increment_x = index as u64 & 1;
        let increment_y = (index as u64 & 2) >> 1;
        let increment_z = (index as u64 & 4) >> 2;

        // and return
        Self {
            x: x + increment_x,
            y: y + increment_y,
            z: z + increment_z,
            depth: self.depth + 1,
        }
    }

    #[inline]
    fn can_subdivide(self, node: Self, detail: u64) -> bool {
        // return early if the level of this chunk is too high
        if node.depth >= self.depth {
            return false;
        }

        // difference in lod level between the target and the node
        let level_difference = self.depth - node.depth;

        // minimum corner of the bounding box
        let min = (
            (node.x << (level_difference + 1))
                .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
            (node.y << (level_difference + 1))
                .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
            (node.z << (level_difference + 1))
                .saturating_sub(((detail + 1) << level_difference) - (1 << level_difference)),
        );

        // max as well
        let max = (
            (node.x << (level_difference + 1))
                .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
            (node.y << (level_difference + 1))
                .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
            (node.z << (level_difference + 1))
                .saturating_add(((detail + 1) << level_difference) + (1 << level_difference)),
        );

        // local position of the target
        let local = (self.x << 1, self.y << 1, self.z << 1);

        // check if the target is inside of the bounding box
        local.0 >= min.0
            && local.0 < max.0
            && local.1 >= min.1
            && local.1 < max.1
            && local.2 >= min.2
            && local.2 < max.2
    }

    fn is_inside_bounds(self, min: Self, max: Self, max_depth: u64) -> bool {
        // get the lowest lod level
        let level = self.depth.min(min.depth.min(max.depth));

        // bring all coords to the lowest level
        let self_difference = self.depth - level;
        let min_difference = min.depth - level;
        let max_difference = max.depth - level;

        // get the coords to that level
        let self_x = self.x >> self_difference;
        let self_y = self.y >> self_difference;
        let self_z = self.z >> self_difference;

        let min_x = min.x >> min_difference;
        let min_y = min.y >> min_difference;
        let min_z = min.z >> min_difference;

        let max_x = max.x >> max_difference;
        let max_y = max.y >> max_difference;
        let max_z = max.z >> max_difference;

        // then check if we are inside the AABB
        self.depth as u64 <= max_depth
            && self_x >= min_x
            && self_x < max_x
            && self_y >= min_y
            && self_y < max_y
            && self_z >= min_z
            && self_z < max_z
    }

    #[inline]
    fn contains_child_node(self, child: Self) -> bool {
        // basically, move the child node up to this level and check if they're equal
        let level_difference = child.depth - self.depth;

        // and move
        let x = child.x >> level_difference;
        let y = child.y >> level_difference;
        let z = child.z >> level_difference;

        // and check
        self.x == x && self.y == y && self.z == z
    }
}
