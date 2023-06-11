use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

// Implements a cardinal neighbour quadtree
#[derive(Clone, Debug)]
pub struct TerrainQuadtree {
    max_depth: u8,
    bounds: AABB,
    root: TerrainQuadtreeNode,
    internals: Vec<TerrainQuadtreeInternal>,
    leaves: Vec<TerrainQuadtreeLeaf>,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TerrainQuadtreeNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
    /// Identifier of a leaf node.
    Leaf(usize),
}

#[derive(Clone, PartialEq)]
struct TerrainQuadtreeInternal {
    parent: TerrainQuadtreeNode,
    bounds: AABB,
    level: u8,
    location: Quadrant,
    children: NodeChildren,
    neighbours: NodeNeighbours,
}

#[derive(Clone, Copy, PartialEq)]
pub struct TerrainQuadtreeLeaf {
    parent: TerrainQuadtreeNode,
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
    north: TerrainQuadtreeNode,
    east: TerrainQuadtreeNode,
    south: TerrainQuadtreeNode,
    west: TerrainQuadtreeNode,
}

impl Default for NodeNeighbours {
    fn default() -> Self {
        Self {
            north: TerrainQuadtreeNode::None,
            east: TerrainQuadtreeNode::None,
            south: TerrainQuadtreeNode::None,
            west: TerrainQuadtreeNode::None,
        }
    }
}

impl Index<Direction> for NodeNeighbours {
    type Output = TerrainQuadtreeNode;

    fn index(&self, dir: Direction) -> &TerrainQuadtreeNode {
        match dir {
            Direction::North => &self.north,
            Direction::East => &self.east,
            Direction::South => &self.south,
            Direction::West => &self.west,
        }
    }
}

impl IndexMut<Direction> for NodeNeighbours {
    fn index_mut(&mut self, dir: Direction) -> &mut TerrainQuadtreeNode {
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
    northwest: TerrainQuadtreeNode,
    northeast: TerrainQuadtreeNode,
    southwest: TerrainQuadtreeNode,
    southeast: TerrainQuadtreeNode,
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

impl TerrainQuadtree {
    pub fn new(camera: [f32; 2], bounds: AABB, max_depth: u8) -> Self {
        #[cfg(feature = "profiler")]
        profile_scope!("quadtree_new");

        let mut tree = Self {
            max_depth: max_depth,
            bounds: bounds,
            root: TerrainQuadtreeNode::None,
            internals: Vec::<TerrainQuadtreeInternal>::new(),
            leaves: Vec::<TerrainQuadtreeLeaf>::new(),
        };

        tree.rebuild(camera);

        tree.internals.shrink_to_fit();
        tree.leaves.shrink_to_fit();

        tree
    }
    // Rebuilds the quadtree based on camera position.
    pub fn rebuild(&mut self, camera: [f32; 2]) {
        if self.root == TerrainQuadtreeNode::None {
            self.root = self.new_node(
                0,
                self.bounds,
                Quadrant::Root,
                TerrainQuadtreeNode::None,
                camera,
            );
            self.subdivide(self.root, camera);
        }

        // Todo: if <camera is too far away from center of leaf> {
        //     self.internals.clear();
        //     self.leaves.clear();
        //     self.root = TerrainQuadtreeNode::None;
        //     self.root = self.build(0, camera, Quadrant::Root, Default::default());
        // }
    }

    #[inline]
    pub fn leaves(&self) -> &[TerrainQuadtreeLeaf] {
        &self.leaves
    }

    #[inline]
    pub fn leaf(&self, index: usize) -> &TerrainQuadtreeLeaf {
        &self.leaves[index]
    }

    pub(crate) fn set_parent(&mut self, node: TerrainQuadtreeNode, parent: TerrainQuadtreeNode) {
        use TerrainQuadtreeNode::*;
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
    pub(crate) fn get_parent(&self, node: TerrainQuadtreeNode) -> TerrainQuadtreeNode {
        match node {
            TerrainQuadtreeNode::Internal(index) => self.internals[index].parent,
            TerrainQuadtreeNode::Leaf(index) => self.leaves[index].parent,
            TerrainQuadtreeNode::None => node,
        }
    }
    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub(crate) fn get_kth_parent(&self, node: TerrainQuadtreeNode, k: u8) -> TerrainQuadtreeNode {
        let mut parent;

        match node {
            TerrainQuadtreeNode::Internal(index) => parent = self.internals[index].parent,
            TerrainQuadtreeNode::Leaf(index) => parent = self.leaves[index].parent,
            _ => unreachable!(),
        }

        for i in 1..k {
            match parent {
                TerrainQuadtreeNode::Internal(index) => parent = self.internals[index].parent,
                TerrainQuadtreeNode::None => return parent,
                _ => unreachable!(),
            }
        }
        parent
    }

    pub(crate) fn get_neighbour(
        &self,
        node: TerrainQuadtreeNode,
        dir: Direction,
    ) -> TerrainQuadtreeNode {
        match node {
            TerrainQuadtreeNode::Internal(index) => {
                let node = &self.internals[index];
                node.neighbours[dir]
            }
            TerrainQuadtreeNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.neighbours[dir]
            }
            _ => unreachable!(),
        }
    }
    fn set_neighbour(
        &mut self,
        node: TerrainQuadtreeNode,
        dir: Direction,
        neighbour: TerrainQuadtreeNode,
    ) {
        match node {
            TerrainQuadtreeNode::Internal(index) => {
                self.internals[index].neighbours[dir] = neighbour
            }
            TerrainQuadtreeNode::Leaf(index) => self.leaves[index].neighbours[dir] = neighbour,
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_neighbours(&self, node: TerrainQuadtreeNode) -> NodeNeighbours {
        match node {
            TerrainQuadtreeNode::Internal(index) => {
                let node = &self.internals[index];
                node.neighbours
            }
            TerrainQuadtreeNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.neighbours
            }
            _ => unreachable!(),
        }
    }

    pub fn get_level(&self, node: TerrainQuadtreeNode) -> u8 {
        match node {
            TerrainQuadtreeNode::Internal(index) => {
                let node = &self.internals[index];
                node.level
            }
            TerrainQuadtreeNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.level
            }
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_bounds(&self, node: TerrainQuadtreeNode) -> AABB {
        match node {
            TerrainQuadtreeNode::Internal(index) => {
                let node = &self.internals[index];
                node.bounds
            }
            TerrainQuadtreeNode::Leaf(index) => {
                let node = &self.leaves[index];
                node.bounds
            }
            _ => unreachable!(),
        }
    }

    pub fn get_neighbour_leaves(&self, node: TerrainQuadtreeNode) -> [TerrainQuadtreeLeaf; 4] {
        unimplemented!()
    }

    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub fn get_kth_neighbour(
        &self,
        node: TerrainQuadtreeNode,
        dir: Direction,
        k: u8,
    ) -> TerrainQuadtreeNode {
        let mut neighbour;
        if k == 0 {
            return node;
        }
        neighbour = self.get_neighbour(node, dir);
        dbg!(dir);
        for _i in 1..k {
            match neighbour {
                TerrainQuadtreeNode::Internal(index) => {
                    neighbour = self.internals[index].neighbours[dir]
                }
                TerrainQuadtreeNode::Leaf(index) => neighbour = self.leaves[index].neighbours[dir],
                TerrainQuadtreeNode::None => return neighbour,
            }
        }
        neighbour
    }

    // Update the NE and SW children of parent.
    pub(crate) fn update_northeast(&mut self, node_id: TerrainQuadtreeNode) {
        if let TerrainQuadtreeNode::Internal(index) = node_id {
            let node = &self.internals[index];
            if node.parent == TerrainQuadtreeNode::None
                || node.neighbours.north == TerrainQuadtreeNode::None
            {
                // We are at the north border
                return;
            }

            let north = node.neighbours.north;
            let northeast = node.children.northeast;
            let bounds = self.get_bounds(northeast);
            if north != TerrainQuadtreeNode::None {
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

    pub(crate) fn update_southwest(&mut self, node_id: TerrainQuadtreeNode) {
        if let TerrainQuadtreeNode::Internal(index) = node_id {
            let node = &self.internals[index];
            if node.parent == TerrainQuadtreeNode::None
                || node.neighbours.west == TerrainQuadtreeNode::None
            {
                // We are at the west border
                return;
            }

            let west = node.neighbours.west;
            let southwest = node.children.southwest;
            let bounds = self.get_bounds(southwest);
            if west != TerrainQuadtreeNode::None {
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

    pub(crate) fn update_neighbours_west(
        &mut self,
        node_id: TerrainQuadtreeNode,
        nw: TerrainQuadtreeNode,
        sw: TerrainQuadtreeNode,
    ) {
        let dir = Direction::West;
        let mut western_id;

        western_id = self.get_neighbour(node_id, dir);
        if western_id == TerrainQuadtreeNode::None {
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
            if western_id != TerrainQuadtreeNode::None
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
        parent_id: TerrainQuadtreeNode,
        ne: TerrainQuadtreeNode,
        nw: TerrainQuadtreeNode,
    ) {
        let dir = Direction::North;
        let mut northern_id;

        northern_id = self.get_neighbour(parent_id, dir);
        if northern_id == TerrainQuadtreeNode::None {
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
            if northern_id != TerrainQuadtreeNode::None {
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

    pub(crate) fn update_neighbours(&mut self, node_id: TerrainQuadtreeNode) {
        match node_id {
            TerrainQuadtreeNode::Internal(index) => {
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

                if north != TerrainQuadtreeNode::None {
                    self.update_neighbours_north(node_id, ne, nw);
                }

                if west != TerrainQuadtreeNode::None {
                    self.update_neighbours_west(node_id, nw, sw);
                }

                if east != TerrainQuadtreeNode::None
                    && self.get_neighbour(east, Direction::West) == node_id
                {
                    self.set_neighbour(east, Direction::West, ne);
                }

                if south != TerrainQuadtreeNode::None
                    && self.get_neighbour(south, Direction::North) == node_id
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
        parent: TerrainQuadtreeNode,
        camera: [f32; 2],
    ) -> TerrainQuadtreeNode {
        if self.split_check(bounds, depth, camera) {
            self.internals.push(TerrainQuadtreeInternal {
                parent: parent,
                bounds: bounds,
                level: depth,
                location: location,
                children: Default::default(),
                neighbours: Default::default(),
            });
            TerrainQuadtreeNode::Internal(self.internals.len() - 1)
        } else {
            self.leaves.push(TerrainQuadtreeLeaf {
                parent: parent,
                bounds: bounds,
                level: depth,
                location: location,
                neighbours: Default::default(),
            });
            TerrainQuadtreeNode::Leaf(self.leaves.len() - 1)
        }
    }

    pub(crate) fn subdivide(&mut self, node_id: TerrainQuadtreeNode, camera: [f32; 2]) {
        if let TerrainQuadtreeNode::Internal(index) = node_id {
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

impl TerrainQuadtreeInternal {
    pub fn contains_point(&self, point: [f32; 2]) -> bool {
        self.bounds.contains_point(point)
    }
}
impl TerrainQuadtreeLeaf {
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

impl fmt::Debug for TerrainQuadtreeInternal {
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
impl fmt::Debug for TerrainQuadtreeLeaf {
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

// impl fmt::Display for TerrainQuadtreeInternal {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if let Some(parent) = self.parent{
//             write!(f, "\nInternal: \nParent: {:?} \nAABB: {},{} ->{},{} \nLevel: {} \nChildren: {:?}, {:?}, {:?}, {:?}\n{:?}\n\n", parent, self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.children[Quadrant::Northwest], self.children[Quadrant::Northeast], self.children[Quadrant::Southwest], self.children[Quadrant::Southeast], self.neighbours)
//         } else {
//             write!(f, "\nRoot Level Internal: \nAABB: {},{} ->{},{} \nLevel: {} \nChildren: {:?}, {:?}, {:?}, {:?}\n\n",  self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.children[Quadrant::Northwest], self.children[Quadrant::Northeast], self.children[Quadrant::Southwest], self.children[Quadrant::Southeast])
//         }
//     }
// }
// impl fmt::Display for TerrainQuadtreeLeaf {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if let Some(parent) = self.parent{
//             write!(f, "\nLeaf: \nParent: {:?} \nAABB: {},{} -> {},{} \nLevel: {}\n{:?}\n{:?}\n\n", parent, self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level, self.location, self.neighbours)
//         } else {
//             write!(f, "\nRoot Level Leaf: \nAABB: {},{} -> {},{} \nLevel: {}\n\n",  self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1], self.level)
//         }
//     }
// }
