use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

// Implements a cardinal neighbour quadtree
#[derive(Clone, Debug)]
pub struct TileTree {
    max_depth: u8,
    bounds: AABB,
    root: TileNode,
    internals: Vec<TileNodeInternal>,
    leaves: Vec<TileTreeLeaf>,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
    /// Identifier of a leaf node.
    Leaf(usize),
}

#[derive(Clone, PartialEq)]
struct TileNodeInternal {
    parent: TileNode,
    bounds: AABB,
    level: u8,
    location: Quadrant,
    children: NodeChildren,
    neighbours: NodeNeighbours,
}

#[derive(Clone, Copy, PartialEq)]
pub struct TileTreeLeaf {
    parent: TileNode,
    bounds: AABB,
    level: u8,
    location: Quadrant,
    neighbours: NodeNeighbours,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct AABB {
    min: [f32; 2],
    max: [f32; 2],
}

impl AABB {
    pub fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        Self { min: min, max: max }
    }
    fn center(&self) -> [f32; 2] {
        let half = self.half_extents();
        [self.min[0] + half[0], self.min[1] + half[1]]
    }
    fn half_extents(&self) -> [f32; 2] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
        ]
    }
    fn contains_point(&self, point: [f32; 2]) -> bool {
        if point[0] < self.min[0]
            || point[1] < self.min[1]
            || point[0] > self.max[0]
            || point[1] > self.max[1]
        {
            return false;
        }
        true
    }
}

impl From<[f32; 4]> for AABB {
    fn from(bounds: [f32; 4]) -> Self {
        Self {
            min: [bounds[0], bounds[1]],
            max: [bounds[2], bounds[3]],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root,
}
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
impl Direction {
    pub(crate) fn traversal(&self) -> Direction {
        match self {
            Direction::West => Direction::South,
            Direction::North => Direction::East,
            Direction::East => Direction::North,
            Direction::South => Direction::West,
        }
    }
    pub(crate) fn opposite(&self) -> Direction {
        match self {
            Direction::West => Direction::East,
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct NodeNeighbours {
    north: TileNode,
    east: TileNode,
    south: TileNode,
    west: TileNode,
}

impl Default for NodeNeighbours {
    fn default() -> Self {
        Self {
            north: TileNode::None,
            east: TileNode::None,
            south: TileNode::None,
            west: TileNode::None,
        }
    }
}

impl Index<Direction> for NodeNeighbours {
    type Output = TileNode;

    fn index(&self, dir: Direction) -> &TileNode {
        match dir {
            Direction::North => &self.north,
            Direction::East => &self.east,
            Direction::South => &self.south,
            Direction::West => &self.west,
        }
    }
}

impl IndexMut<Direction> for NodeNeighbours {
    fn index_mut(&mut self, dir: Direction) -> &mut TileNode {
        match dir {
            Direction::North => &mut self.north,
            Direction::East => &mut self.east,
            Direction::South => &mut self.south,
            Direction::West => &mut self.west,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct NodeChildren {
    northwest: TileNode,
    northeast: TileNode,
    southwest: TileNode,
    southeast: TileNode,
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

impl TileTree {
    pub fn new(camera: [f32; 2], bounds: AABB, max_depth: u8) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("quadtree_new");

        let mut tree = Self {
            max_depth: max_depth,
            bounds: bounds,
            root: TileNode::None,
            internals: Vec::<TileNodeInternal>::new(),
            leaves: Vec::<TileTreeLeaf>::new(),
        };

        tree.rebuild(camera);

        tree.internals.shrink_to_fit();
        tree.leaves.shrink_to_fit();

        tree
    }
    // Rebuilds the quadtree based on camera position.
    pub fn rebuild(&mut self, camera: [f32; 2]) {
        if self.root == TileNode::None {
            self.root = self.new_node(0, self.bounds, Quadrant::Root, TileNode::None, camera);
            self.subdivide(self.root, camera);
        }

        // Todo: if <camera is too far away from center of leaf> {
        //     self.internals.clear();
        //     self.leaves.clear();
        //     self.root = TileNode::None;
        //     self.root = self.build(0, camera, Quadrant::Root, Default::default());
        // }
    }

    #[inline]
    pub fn leaves(&self) -> &[TileTreeLeaf] {
        &self.leaves
    }

    #[inline]
    pub fn leaf(&self, index: usize) -> &TileTreeLeaf {
        &self.leaves[index]
    }

    pub(crate) fn set_parent(&mut self, node: TileNode, parent: TileNode) {
        use TileNode::*;
        match node {
            Internal(index) => {
                self.internals[index].parent = parent;
            }
            Leaf(index) => {
                self.leaves[index].parent = parent;
            }
            _ => unreachable!(),
        }
    }

    // Returns Parent of node or node if node is root
    pub(crate) fn get_parent(&self, node: TileNode) -> TileNode {
        match node {
            TileNode::Internal(index) => self.internals[index].parent,
            TileNode::Leaf(index) => self.leaves[index].parent,
            TileNode::None => node,
        }
    }
    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub(crate) fn get_kth_parent(&self, node: TileNode, k: u8) -> TileNode {
        let mut parent;

        match node {
            TileNode::Internal(index) => parent = self.internals[index].parent,
            TileNode::Leaf(index) => parent = self.leaves[index].parent,
            _ => unreachable!(),
        }

        for i in 1..k {
            match parent {
                TileNode::Internal(index) => parent = self.internals[index].parent,
                TileNode::None => return parent,
                _ => unreachable!(),
            }
        }
        parent
    }

    pub(crate) fn get_neighbour(&self, node: TileNode, dir: Direction) -> TileNode {
        match node {
            TileNode::Internal(index) => {
                let node = &self.internals[index];
                node.neighbours[dir]
            }
            TileNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.neighbours[dir]
            }
            _ => unreachable!(),
        }
    }
    fn set_neighbour(&mut self, node: TileNode, dir: Direction, neighbour: TileNode) {
        match node {
            TileNode::Internal(index) => self.internals[index].neighbours[dir] = neighbour,
            TileNode::Leaf(index) => self.leaves[index].neighbours[dir] = neighbour,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_neighbours(&self, node: TileNode) -> NodeNeighbours {
        match node {
            TileNode::Internal(index) => {
                let node = &self.internals[index];
                node.neighbours
            }
            TileNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.neighbours
            }
            _ => unreachable!(),
        }
    }

    pub fn get_level(&self, node: TileNode) -> u8 {
        match node {
            TileNode::Internal(index) => {
                let node = &self.internals[index];
                node.level
            }
            TileNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.level
            }
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_bounds(&self, node: TileNode) -> AABB {
        match node {
            TileNode::Internal(index) => {
                let node = &self.internals[index];
                node.bounds
            }
            TileNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.bounds
            }
            _ => unreachable!(),
        }
    }

    pub fn get_neighbour_leaves(&self, node: TileNode) -> [TileTreeLeaf; 4] {
        unimplemented!()
    }

    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub fn get_kth_neighbour(&self, node: TileNode, dir: Direction, k: u8) -> TileNode {
        let mut neighbour;
        if k == 0 {
            return node;
        }
        neighbour = self.get_neighbour(node, dir);
        dbg!(dir);
        for _i in 1..k {
            match neighbour {
                TileNode::Internal(index) => neighbour = self.internals[index].neighbours[dir],
                TileNode::Leaf(index) => neighbour = self.leaves[index].neighbours[dir],
                TileNode::None => return neighbour,
            }
        }
        neighbour
    }

    // Update the NE and SW children of parent.
    pub(crate) fn update_northeast(&mut self, node_id: TileNode) {
        if let TileNode::Internal(index) = node_id {
            let node = &self.internals[index];
            if node.parent == TileNode::None || node.neighbours.north == TileNode::None {
                // We are at the north border
                return;
            }

            let north = node.neighbours.north;
            let northeast = node.children.northeast;
            let bounds = self.get_bounds(northeast);
            if north != TileNode::None {
                let north_level = self.get_level(north);

                if north_level > node.level {
                    let nw = node.children.northwest;
                    {
                        self.set_neighbour(nw, Direction::North, north);

                        let mut cur = north;
                        // let mut acc = 1./(2_i32.pow(north_level as u32) as f32);
                        // while acc < 1./(2_i32.pow(self.get_level(nw) as u32) as f32) {
                        let mut nbounds = self.get_bounds(cur);
                        while nbounds.max[0] <= bounds.min[0] {
                            cur = self.get_neighbour(cur, Direction::East);
                            // acc += 1./(2_i32.pow(self.get_level(cur) as u32) as f32);
                            nbounds = self.get_bounds(cur);
                        }

                        self.set_neighbour(northeast, Direction::North, cur);
                    }
                }
            }
        }
    }

    pub(crate) fn update_southwest(&mut self, node_id: TileNode) {
        if let TileNode::Internal(index) = node_id {
            let node = &self.internals[index];
            if node.parent == TileNode::None || node.neighbours.west == TileNode::None {
                // We are at the west border
                return;
            }

            let west = node.neighbours.west;
            let southwest = node.children.southwest;
            let bounds = self.get_bounds(southwest);
            if west != TileNode::None {
                let west_level = self.get_level(west);

                if west_level > node.level {
                    let nw = node.children.northwest;
                    {
                        self.set_neighbour(nw, Direction::North, node.neighbours.north);

                        let mut cur = west;
                        // let mut acc = 1./(2_i32.pow(west_level as u32) as f32);
                        let mut nbounds = self.get_bounds(cur);
                        // while acc < 1./(2_i32.pow(self.get_level(nw) as u32) as f32) {
                        while nbounds.max[1] <= bounds.min[1] {
                            cur = self.get_neighbour(cur, Direction::South);
                            nbounds = self.get_bounds(cur);
                            // acc += 1./(2_i32.pow(self.get_level(cur) as u32) as f32);
                        }

                        self.set_neighbour(southwest, Direction::West, cur);
                    }
                }
            }
        }
    }

    pub(crate) fn update_neighbours_west(&mut self, node_id: TileNode, nw: TileNode, sw: TileNode) {
        let dir = Direction::West;
        let mut western_id;

        western_id = self.get_neighbour(node_id, dir);
        if western_id == TileNode::None {
            return;
        }
        let opposite = self.get_neighbour(western_id, dir.opposite());
        if opposite == node_id {
            let child_bounds = self.get_bounds(sw);
            let neighbour_bounds = self.get_bounds(opposite);
            let western_bounds = &self.get_bounds(western_id);
            if western_bounds.max[1] > child_bounds.min[1] {
                self.set_neighbour(western_id, dir.opposite(), sw);
            } else {
                self.set_neighbour(western_id, dir.opposite(), nw);
            }
            if western_bounds.min[1] == neighbour_bounds.min[1] {
                self.set_neighbour(opposite, Direction::West, western_id);
            }
        }
        if self.get_level(western_id) <= self.get_level(node_id) {
            return;
        }

        loop {
            western_id = self.get_neighbour(western_id, dir.traversal());
            if western_id != TileNode::None
                && self.get_neighbour(western_id, dir.opposite()) == node_id
            {
                let opposite = self.get_neighbour(western_id, dir.opposite());
                if opposite == node_id {
                    let child_bounds = self.get_bounds(sw);
                    let neighbour_bounds = self.get_bounds(opposite);
                    let western_bounds = &self.get_bounds(western_id);
                    if western_bounds.max[1] > child_bounds.min[1] {
                        self.set_neighbour(western_id, dir.opposite(), sw);
                    } else {
                        self.set_neighbour(western_id, dir.opposite(), nw);
                    }
                    if western_bounds.min[1] == neighbour_bounds.min[1] {
                        self.set_neighbour(opposite, dir, western_id);
                    }
                }
            } else {
                break;
            }
        }
    }

    pub(crate) fn update_neighbours_north(
        &mut self,
        parent_id: TileNode,
        ne: TileNode,
        nw: TileNode,
    ) {
        let dir = Direction::North;
        let mut northern_id;

        northern_id = self.get_neighbour(parent_id, dir);
        if northern_id == TileNode::None {
            return;
        }
        let opposite = self.get_neighbour(northern_id, dir.opposite());
        if opposite == parent_id {
            let child_bounds = self.get_bounds(ne);
            let neighbour_bounds = self.get_bounds(opposite);
            let northern_bounds = &self.get_bounds(northern_id);
            if northern_bounds.max[0] > child_bounds.min[0] {
                self.set_neighbour(northern_id, dir.opposite(), ne);
            } else {
                self.set_neighbour(northern_id, dir.opposite(), nw);
            }
            if northern_bounds.min[0] == neighbour_bounds.min[0] {
                self.set_neighbour(opposite, dir, northern_id);
            }
        }
        if self.get_level(northern_id) <= self.get_level(parent_id) {
            return;
        }

        loop {
            northern_id = self.get_neighbour(northern_id, dir.traversal());
            if northern_id != TileNode::None {
                let opposite = self.get_neighbour(northern_id, dir.opposite());
                if opposite == parent_id {
                    if opposite == parent_id {
                        let child_bounds = self.get_bounds(ne);
                        let neighbour_bounds = self.get_bounds(opposite);
                        let northern_bounds = &self.get_bounds(northern_id);
                        if northern_bounds.max[0] > child_bounds.min[0] {
                            self.set_neighbour(northern_id, dir.opposite(), ne);
                        } else {
                            self.set_neighbour(northern_id, dir.opposite(), nw);
                        }
                        if northern_bounds.min[0] == neighbour_bounds.min[0] {
                            self.set_neighbour(opposite, dir, northern_id);
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub(crate) fn update_neighbours(&mut self, node_id: TileNode) {
        match node_id {
            TileNode::Internal(index) => {
                let nw;
                let ne;
                let sw;
                let se;
                let west;
                let north;
                let east;
                let south;
                {
                    let node = &self.internals[index];
                    nw = node.children.northwest;
                    ne = node.children.northeast;
                    sw = node.children.southwest;
                    se = node.children.southeast;
                    west = node.neighbours.west;
                    north = node.neighbours.north;
                    east = node.neighbours.east;
                    south = node.neighbours.south;
                }

                if north != TileNode::None {
                    self.update_neighbours_north(node_id, ne, nw);
                }

                if west != TileNode::None {
                    self.update_neighbours_west(node_id, nw, sw);
                }

                if east != TileNode::None && self.get_neighbour(east, Direction::West) == node_id {
                    self.set_neighbour(east, Direction::West, ne);
                }

                if south != TileNode::None && self.get_neighbour(south, Direction::North) == node_id
                {
                    self.set_neighbour(south, Direction::North, sw);
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn new_node(
        &mut self,
        depth: u8,
        bounds: AABB,
        location: Quadrant,
        parent: TileNode,
        camera: [f32; 2],
    ) -> TileNode {
        if self.split_check(bounds, depth, camera) {
            self.internals.push(TileNodeInternal {
                parent: parent,
                bounds: bounds,
                level: depth,
                location: location,
                children: Default::default(),
                neighbours: Default::default(),
            });
            TileNode::Internal(self.internals.len() - 1)
        } else {
            self.leaves.push(TileTreeLeaf {
                parent: parent,
                bounds: bounds,
                level: depth,
                location: location,
                neighbours: Default::default(),
            });
            TileNode::Leaf(self.leaves.len() - 1)
        }
    }

    pub(crate) fn subdivide(&mut self, node_id: TileNode, camera: [f32; 2]) {
        if let TileNode::Internal(index) = node_id {
            let node = &self.internals[index];
            let bounds = node.bounds;
            let depth = node.level;
            let neighbours = node.neighbours;
            let half_extents = bounds.half_extents();
            let nw_bounds = AABB::new(
                bounds.min,
                [
                    bounds.max[0] - half_extents[0],
                    bounds.max[1] - half_extents[1],
                ],
            );
            let ne_bounds = AABB::new(
                [bounds.min[0] + half_extents[0], bounds.min[1]],
                [bounds.max[0], bounds.max[1] - half_extents[1]],
            );
            let sw_bounds = AABB::new(
                [bounds.min[0], bounds.min[1] + half_extents[1]],
                [bounds.max[0] - half_extents[0], bounds.max[1]],
            );
            let se_bounds = AABB::new(
                [
                    bounds.min[0] + half_extents[0],
                    bounds.min[1] + half_extents[1],
                ],
                bounds.max,
            );

            let nw = self.new_node(depth + 1, nw_bounds, Quadrant::Northwest, node_id, camera);
            let ne = self.new_node(depth + 1, ne_bounds, Quadrant::Northeast, node_id, camera);
            let sw = self.new_node(depth + 1, sw_bounds, Quadrant::Southwest, node_id, camera);
            let se = self.new_node(depth + 1, se_bounds, Quadrant::Southeast, node_id, camera);

            self.set_neighbour(nw, Direction::West, neighbours.west);
            self.set_neighbour(nw, Direction::North, neighbours.north);
            self.set_neighbour(nw, Direction::East, ne);
            self.set_neighbour(nw, Direction::South, sw);

            self.set_neighbour(ne, Direction::West, nw);
            self.set_neighbour(ne, Direction::North, neighbours.north);
            self.set_neighbour(ne, Direction::East, neighbours.east);
            self.set_neighbour(ne, Direction::South, se);

            self.set_neighbour(sw, Direction::West, neighbours.west);
            self.set_neighbour(sw, Direction::North, nw);
            self.set_neighbour(sw, Direction::East, se);
            self.set_neighbour(sw, Direction::South, neighbours.south);

            self.set_neighbour(se, Direction::West, sw);
            self.set_neighbour(se, Direction::North, ne);
            self.set_neighbour(se, Direction::East, neighbours.east);
            self.set_neighbour(se, Direction::South, neighbours.south);

            {
                let node = &mut self.internals[index];
                node.children.northwest = nw;
                node.children.northeast = ne;
                node.children.southwest = sw;
                node.children.southeast = se;
            }

            self.update_northeast(node_id);

            self.update_southwest(node_id);

            self.update_neighbours(node_id);

            self.subdivide(nw, camera);
            self.subdivide(ne, camera);
            self.subdivide(sw, camera);
            self.subdivide(se, camera);
        }
    }

    // Todo: make the distance level a param.
    fn split_check(&self, bounds: AABB, level: u8, camera: [f32; 2]) -> bool {
        // Squared distance from node center to camera
        let center = bounds.center();
        let d = (camera[0] - center[0]).powf(2.) + (camera[1] - center[1]).powf(2.);

        // Check base case:
        // Distance to camera is greater than twice the length of the diagonal
        // from current origin to corner of current square.
        // OR
        // Max recursion level has been hit
        let half = bounds.half_extents();
        if d > 2.5 * (half[0].powf(2.) + half[1].powf(2.)) || level >= self.max_depth {
            return false;
        }
        true
    }
}

impl TileNodeInternal {
    pub fn contains_point(&self, point: [f32; 2]) -> bool {
        self.bounds.contains_point(point)
    }
}
impl TileTreeLeaf {
    pub fn contains_point(&self, point: [f32; 2]) -> bool {
        self.bounds.contains_point(point)
    }

    pub fn origin(self) -> [f32; 2] {
        self.bounds.center()
    }

    pub fn half_extents(self) -> [f32; 2] {
        self.bounds.half_extents()
    }

    pub fn level(self) -> u8 {
        self.level
    }
    pub fn get_neighbours(&self) -> NodeNeighbours {
        self.neighbours
    }

    // TODO: Improve frustum culling
    pub fn check_visibility(self, a: [f32; 2], b: [f32; 2]) -> bool {
        return true;
        let C = self.origin();
        (b[0] - a[0]) * (C[1] - a[1]) - (C[0] - a[0]) * (b[1] - a[1]) >= 0.0
    }
}

impl fmt::Debug for TileNodeInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?}, ({:?}, {:?}), GREY, {:?}, {:?}, ({:?}), ({:?}))",
            self.level,
            self.bounds.min,
            self.bounds.max,
            self.location,
            self.parent,
            self.children,
            self.neighbours,
        )
    }
}
impl fmt::Debug for TileTreeLeaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?}, ({:?}, {:?}), WHITE, {:?}, {:?}, (#, #, #, #), ({:?}))",
            self.level,
            self.bounds.min,
            self.bounds.max,
            self.location,
            self.parent,
            self.neighbours
        )
    }
}

impl fmt::Debug for NodeNeighbours {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}, {:?}",
            self.north, self.east, self.south, self.west
        )
    }
}
impl fmt::Debug for NodeChildren {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}, {:?}",
            self.northwest, self.northeast, self.southwest, self.southeast
        )
    }
}

// impl fmt::Display for TileNodeInternal {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if let Some(parent) = self.parent{
//             write!(f, "\nInternal: \nParent: {:?} \nAABB: {},{} ->{},{} \nLevel: {} \nChildren: {:?}, {:?}, {:?}, {:?}\n{:?}\n\n", parent, self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.children[Quadrant::Northwest], self.children[Quadrant::Northeast], self.children[Quadrant::Southwest], self.children[Quadrant::Southeast], self.neighbours)
//         } else {
//             write!(f, "\nRoot Level Internal: \nAABB: {},{} ->{},{} \nLevel: {} \nChildren: {:?}, {:?}, {:?}, {:?}\n\n",  self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.children[Quadrant::Northwest], self.children[Quadrant::Northeast], self.children[Quadrant::Southwest], self.children[Quadrant::Southeast])
//         }
//     }
// }
// impl fmt::Display for TileTreeLeaf {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if let Some(parent) = self.parent{
//             write!(f, "\nLeaf: \nParent: {:?} \nAABB: {},{} -> {},{} \nLevel: {}\n{:?}\n{:?}\n\n", parent, self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.location, self.neighbours)
//         } else {
//             write!(f, "\nRoot Level Leaf: \nAABB: {},{} -> {},{} \nLevel: {}\n\n",  self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level)
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use TileNode as Node;

    macro_rules! Leaf {
        ($parent:expr, $min:expr, $max:expr, $location:expr, $north:expr, $east:expr, $south:expr, $west:expr, $level:expr) => {
            TileTreeLeaf {
                parent: $parent,
                bounds: AABB::new($min, $max),
                location: $location,
                neighbours: NodeNeighbours {
                    north: $north,
                    east: $east,
                    south: $south,
                    west: $west,
                },
                level: $level,
            }
        };
    }

    #[test]
    fn quadtree_creation_with_depth_0() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 0);
        assert_eq!(tree.root, TileNode::Leaf(0));
        assert_eq!(
            tree.leaves,
            vec![Leaf!(
                Node::None,
                [-100., -100.],
                [100., 100.],
                Quadrant::Root,
                Node::None,
                Node::None,
                Node::None,
                Node::None,
                0
            ),]
        );
    }

    #[test]
    fn quadtree_creation_with_depth_1() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 1);

        assert_eq!(tree.root, TileNode::Internal(0));
        assert_eq!(tree.leaves.len(), 4);
        assert_eq!(
            tree.internals,
            vec![TileNodeInternal {
                parent: Node::None,
                bounds: AABB::new([-100., -100.], [100., 100.]),
                level: 0,
                children: NodeChildren {
                    northwest: Node::Leaf(0),
                    northeast: Node::Leaf(1),
                    southwest: Node::Leaf(2),
                    southeast: Node::Leaf(3)
                },
                location: Quadrant::Root,
                neighbours: NodeNeighbours {
                    north: Node::None,
                    east: Node::None,
                    south: Node::None,
                    west: Node::None
                },
            },]
        );
        assert_eq!(
            tree.leaves,
            vec![
                Leaf!(
                    Node::Internal(0),
                    [-100., -100.],
                    [0., 0.],
                    Quadrant::Northwest,
                    Node::None,
                    Node::Leaf(1),
                    Node::Leaf(2),
                    Node::None,
                    1
                ),
                Leaf!(
                    Node::Internal(0),
                    [0., -100.],
                    [100., 0.],
                    Quadrant::Northeast,
                    Node::None,
                    Node::None,
                    Node::Leaf(3),
                    Node::Leaf(0),
                    1
                ),
                Leaf!(
                    Node::Internal(0),
                    [-100., 0.],
                    [0., 100.],
                    Quadrant::Southwest,
                    Node::Leaf(0),
                    Node::Leaf(3),
                    Node::None,
                    Node::None,
                    1
                ),
                Leaf!(
                    Node::Internal(0),
                    [0., 0.],
                    [100., 100.],
                    Quadrant::Southeast,
                    Node::Leaf(1),
                    Node::None,
                    Node::None,
                    Node::Leaf(2),
                    1
                ),
            ]
        )
    }

    #[test]
    fn quadtree_creation_with_depth_2() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 2);

        assert_eq!(tree.root, TileNode::Internal(0));
        assert_eq!(tree.leaves.len(), 16);
        // Todo: The Neighbours of the internals are not correct. They refer partly to leaves and internals.
        // This seems to not have no effect on leaf neighbour calculation.

        // assert_eq!(tree.internals, vec![
        //     TileNodeInternal{
        //         parent: Node::None,
        //         bounds: AABB::new([-100., -100.], [100., 100.]),
        //         level: 0,
        //         children: NodeChildren{northwest:Node::Internal(1), northeast: Node::Internal(2), southwest: Node::Internal(3), southeast: Node::Internal(4)},
        //         location: Quadrant::Root,
        //         neighbours: NodeNeighbours{north: Node::None, east: Node::None, south: Node::None, west: Node::None},
        //     },
        //     TileNodeInternal{
        //         parent: Node::Internal(0),
        //         bounds: AABB::new([-100., -100.], [0., 0.]),
        //         level: 1,
        //         children: NodeChildren{northwest: Node::Leaf(0), northeast: Node::Leaf(1), southwest: Node::Leaf(2), southeast: Node::Leaf(3)},
        //         location: Quadrant::Northwest,
        //         neighbours: NodeNeighbours{north: Node::None, east: Node::Leaf(6), south: Node::Leaf(9), west: Node::None},
        //     },
        //     TileNodeInternal{
        //         parent: Node::Internal(0),
        //         bounds: AABB::new([0., -100.], [100., 0.]),
        //         level: 1,
        //         children: NodeChildren{northwest: Node::Leaf(4), northeast: Node::Leaf(5), southwest: Node::Leaf(6), southeast: Node::Leaf(7)},
        //         location: Quadrant::Northeast,
        //         neighbours: NodeNeighbours{north: Node::None, east: Node::None, south: Node::Leaf(13), west: Node::Leaf(1)},
        //     },
        //     TileNodeInternal{
        //         parent: Node::Internal(0),
        //         bounds: AABB::new([-100., 0.], [0., 100.]),
        //         level: 1,
        //         children: NodeChildren{northwest:Node::Leaf(8), northeast: Node::Leaf(9), southwest: Node::Leaf(10), southeast: Node::Leaf(11)},
        //         location: Quadrant::Southwest,
        //         neighbours: NodeNeighbours{north: Node::Leaf(2), east: Node::Leaf(14), south: Node::None, west: Node::None},
        //     },
        //     TileNodeInternal{
        //         parent: Node::Internal(0),
        //         bounds: AABB::new([0., 0.], [100., 100.]),
        //         level: 1,
        //         children: NodeChildren{northwest:Node::Leaf(12), northeast: Node::Leaf(13), southwest: Node::Leaf(14), southeast: Node::Leaf(15)},
        //         location: Quadrant::Southeast,
        //         neighbours: NodeNeighbours{north: Node::Leaf(6), east: Node::None, south: Node::None, west: Node::Leaf(9)},
        //     },

        // ]);
        let leaf_ref = vec![
            Leaf!(
                Node::Internal(1),
                [-100., -100.],
                [-50., -50.],
                Quadrant::Northwest,
                Node::None,
                Node::Leaf(1),
                Node::Leaf(2),
                Node::None,
                2
            ), // 0
            Leaf!(
                Node::Internal(1),
                [-50., -100.],
                [0., -50.],
                Quadrant::Northeast,
                Node::None,
                Node::Leaf(4),
                Node::Leaf(3),
                Node::Leaf(0),
                2
            ), // 1
            Leaf!(
                Node::Internal(1),
                [-100., -50.],
                [-50., 0.],
                Quadrant::Southwest,
                Node::Leaf(0),
                Node::Leaf(3),
                Node::Leaf(8),
                Node::None,
                2
            ), // 2
            Leaf!(
                Node::Internal(1),
                [-50., -50.],
                [0., 0.],
                Quadrant::Southeast,
                Node::Leaf(1),
                Node::Leaf(6),
                Node::Leaf(9),
                Node::Leaf(2),
                2
            ), // 3
            Leaf!(
                Node::Internal(2),
                [0., -100.],
                [50., -50.],
                Quadrant::Northwest,
                Node::None,
                Node::Leaf(5),
                Node::Leaf(6),
                Node::Leaf(1),
                2
            ), // 4
            Leaf!(
                Node::Internal(2),
                [50., -100.],
                [100., -50.],
                Quadrant::Northeast,
                Node::None,
                Node::None,
                Node::Leaf(7),
                Node::Leaf(4),
                2
            ), // 5
            Leaf!(
                Node::Internal(2),
                [0., -50.],
                [50., 0.],
                Quadrant::Southwest,
                Node::Leaf(4),
                Node::Leaf(7),
                Node::Leaf(12),
                Node::Leaf(3),
                2
            ), // 6
            Leaf!(
                Node::Internal(2),
                [50., -50.],
                [100., 0.],
                Quadrant::Southeast,
                Node::Leaf(5),
                Node::None,
                Node::Leaf(13),
                Node::Leaf(6),
                2
            ), // 7
            Leaf!(
                Node::Internal(3),
                [-100., 0.],
                [-50., 50.],
                Quadrant::Northwest,
                Node::Leaf(2),
                Node::Leaf(9),
                Node::Leaf(10),
                Node::None,
                2
            ), // 8
            Leaf!(
                Node::Internal(3),
                [-50., 0.],
                [0., 50.],
                Quadrant::Northeast,
                Node::Leaf(3),
                Node::Leaf(12),
                Node::Leaf(11),
                Node::Leaf(8),
                2
            ), // 9
            Leaf!(
                Node::Internal(3),
                [-100., 50.],
                [-50., 100.],
                Quadrant::Southwest,
                Node::Leaf(8),
                Node::Leaf(11),
                Node::None,
                Node::None,
                2
            ), // 10
            Leaf!(
                Node::Internal(3),
                [-50., 50.],
                [0., 100.],
                Quadrant::Southeast,
                Node::Leaf(9),
                Node::Leaf(6),
                Node::None,
                Node::Leaf(10),
                2
            ), // 11
            Leaf!(
                Node::Internal(4),
                [0., 0.],
                [50., 50.],
                Quadrant::Northwest,
                Node::Leaf(6),
                Node::Leaf(13),
                Node::Leaf(14),
                Node::Leaf(9),
                2
            ), // 12
            Leaf!(
                Node::Internal(4),
                [50., 0.],
                [100., 50.],
                Quadrant::Northeast,
                Node::Leaf(7),
                Node::None,
                Node::Leaf(15),
                Node::Leaf(12),
                2
            ), // 13
            Leaf!(
                Node::Internal(4),
                [0., 50.],
                [50., 100.],
                Quadrant::Southwest,
                Node::Leaf(12),
                Node::Leaf(15),
                Node::None,
                Node::Leaf(11),
                2
            ), // 14
            Leaf!(
                Node::Internal(4),
                [50., 50.],
                [100., 100.],
                Quadrant::Southeast,
                Node::Leaf(13),
                Node::None,
                Node::None,
                Node::Leaf(14),
                2
            ), // 15
        ];

        assert_eq!(&tree.leaves[0..4], &leaf_ref[0..4]);
        assert_eq!(&tree.leaves[4..8], &leaf_ref[4..8]);
        assert_eq!(&tree.leaves[8..11], &leaf_ref[8..11]);
        assert_eq!(&tree.leaves[12..16], &leaf_ref[12..16]);
    }

    #[test]
    fn quadtree_creation_with_depth_3_asym() {
        let tree = TileTree::new([-50., -50.], AABB::new([-100., -100.], [100., 100.]), 3);

        assert_eq!(tree.root, TileNode::Internal(0));
        assert_eq!(tree.leaves.len(), 25);
        let leaf_ref = vec![
            Leaf!(
                Node::Internal(4),
                [-100., -100.],
                [-75., -75.],
                Quadrant::Northwest,
                Node::None,
                Node::Leaf(2),
                Node::Leaf(3),
                Node::None,
                3
            ), // 000
            Leaf!(
                Node::Internal(4),
                [-75., -100.],
                [-50., -75.],
                Quadrant::Northeast,
                Node::None,
                Node::Leaf(5),
                Node::Leaf(4),
                Node::Leaf(1),
                3
            ), // 001
            Leaf!(
                Node::Internal(4),
                [-100., -75.],
                [-75., -50.],
                Quadrant::Southwest,
                Node::Leaf(1),
                Node::Leaf(4),
                Node::Leaf(9),
                Node::None,
                3
            ), // 002
            Leaf!(
                Node::Internal(4),
                [-75., -75.],
                [-50., -50.],
                Quadrant::Southeast,
                Node::Leaf(2),
                Node::Leaf(7),
                Node::Leaf(10),
                Node::Leaf(3),
                3
            ), // 003
            Leaf!(
                Node::Internal(5),
                [-25., -100.],
                [0., -75.],
                Quadrant::Northeast,
                Node::None,
                Node::Leaf(17),
                Node::Leaf(8),
                Node::Leaf(5),
                3
            ), // 011
            Leaf!(
                Node::Internal(6),
                [-100., -50.],
                [-75., -25.],
                Quadrant::Northwest,
                Node::Leaf(3),
                Node::Leaf(10),
                Node::Leaf(11),
                Node::None,
                3
            ), // 020
            Leaf!(
                Node::Internal(6),
                [-100., -25.],
                [-75., 0.],
                Quadrant::Southwest,
                Node::Leaf(9),
                Node::Leaf(12),
                Node::Leaf(21),
                Node::None,
                3
            ), // 022
            Leaf!(
                Node::Internal(7),
                [-50., -50.],
                [-25., -25.],
                Quadrant::Northwest,
                Node::Leaf(7),
                Node::Leaf(14),
                Node::Leaf(15),
                Node::Leaf(10),
                3
            ), // 030
            Leaf!(
                Node::Internal(7),
                [-25., -50.],
                [0., -25.],
                Quadrant::Northeast,
                Node::Leaf(8),
                Node::Leaf(19),
                Node::Leaf(16),
                Node::Leaf(13),
                3
            ), // 031
            Leaf!(
                Node::Internal(7),
                [-50., -25.],
                [-25., 0.],
                Quadrant::Southwest,
                Node::Leaf(13),
                Node::Leaf(16),
                Node::Leaf(22),
                Node::Leaf(12),
                3
            ), // 032
            Leaf!(
                Node::Internal(7),
                [-25., -25.],
                [0., 0.],
                Quadrant::Southeast,
                Node::Leaf(14),
                Node::Leaf(19),
                Node::Leaf(22),
                Node::Leaf(15),
                3
            ), // 033
        ];

        let leaf_ref_10 = Leaf!(
            Node::Internal(2),
            [0., -100.],
            [50., -50.],
            Quadrant::Northwest,
            Node::None,
            Node::Leaf(18),
            Node::Leaf(19),
            Node::Leaf(6),
            2
        );
        let leaf_ref_12 = Leaf!(
            Node::Internal(2),
            [0., -50.],
            [50., 0.],
            Quadrant::Southwest,
            Node::Leaf(17),
            Node::Leaf(20),
            Node::Leaf(0),
            Node::Leaf(14),
            2
        );

        let leaf_ref_21 = Leaf!(
            Node::Internal(3),
            [-50., 0.],
            [0., 50.],
            Quadrant::Northeast,
            Node::Leaf(15),
            Node::Leaf(0),
            Node::Leaf(24),
            Node::Leaf(21),
            2
        );
        let leaf_ref_23 = Leaf!(
            Node::Internal(3),
            [-50., 50.],
            [0., 100.],
            Quadrant::Southeast,
            Node::Leaf(22),
            Node::Leaf(0),
            Node::None,
            Node::Leaf(23),
            2
        );

        let leaf_ref_3 = Leaf!(
            Node::Internal(0),
            [0., 00.],
            [100., 100.],
            Quadrant::Southeast,
            Node::Leaf(19),
            Node::None,
            Node::None,
            Node::Leaf(22),
            1
        );

        assert_eq!(&tree.leaves[1..5], &leaf_ref[0..4]);
        assert_eq!(&tree.leaves[6], &leaf_ref[4]);
        assert_eq!(&tree.leaves[9], &leaf_ref[5]);
        assert_eq!(&tree.leaves[11], &leaf_ref[6]);
        assert_eq!(&tree.leaves[13..17], &leaf_ref[7..11]);

        assert_eq!(&tree.leaves[17], &leaf_ref_10);
        assert_eq!(&tree.leaves[19], &leaf_ref_12);
        assert_eq!(&tree.leaves[22], &leaf_ref_21);
        assert_eq!(&tree.leaves[24], &leaf_ref_23);
        assert_eq!(&tree.leaves[0], &leaf_ref_3);
    }

    #[test]
    fn get_kth_neighbour_with_distance_0() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 1);
        let left = tree.get_kth_neighbour(TileNode::Leaf(0), Direction::East, 0);

        assert_eq!(left, TileNode::Leaf(0))
    }
    #[test]
    fn get_kth_neighbour_with_distance_1() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 1);
        let left = tree.get_kth_neighbour(TileNode::Leaf(0), Direction::East, 1);

        assert_eq!(left, TileNode::Leaf(1))
    }
    #[test]
    fn get_kth_neighbour_with_distance_2() {
        let tree = TileTree::new([0., 0.], AABB::new([-100., -100.], [100., 100.]), 2);
        let left = tree.get_kth_neighbour(TileNode::Leaf(1), Direction::East, 2);

        assert_eq!(left, TileNode::Leaf(5))
    }

    #[test]
    fn get_kth_neighbour_with_distance_3_asym() {
        let tree = TileTree::new([-50., -50.], AABB::new([-100., -100.], [100., 100.]), 3);
        let left = tree.get_kth_neighbour(TileNode::Leaf(10), Direction::East, 3);

        assert_eq!(left, TileNode::Leaf(19))
    }
}
