use houtu_scene::{GeographicTilingScheme, TilingScheme};

use super::{
    direction::Direction, node_id::NodeId, terrain_quadtree_internal::TerrainQuadtreeInternal,
    terrain_quadtree_leaf::TerrainQuadtreeLeaf, terrain_quadtree_node::TerrainQuadtreeNode,
    Quadrant,
};

pub struct TerrainQuadtree {
    max_depth: u8,
    root: TerrainQuadtreeNode,
    pub(super) internals: Vec<TerrainQuadtreeInternal>,
    pub(super) leaves: Vec<TerrainQuadtreeLeaf>,
    tiling_scheme: GeographicTilingScheme,
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
        node_id: NodeId,
        location: Quadrant,
        parent: TerrainQuadtreeNode,
    ) -> TerrainQuadtreeNode {
        if self.split_check(bounds, depth, camera) {
            self.internals.push(TerrainQuadtreeInternal {
                parent: parent,
                id: node_id,
                rectangle: self.tiling_scheme.tile_x_y_to_rectange(
                    node_id.x,
                    node_id.y,
                    node_id.level,
                ),
                children: Default::default(),
                neighbours: Default::default(),
                location: location,
            });
            TerrainQuadtreeNode::Internal(self.internals.len() - 1)
        } else {
            self.leaves.push(TerrainQuadtreeLeaf {
                parent: parent,
                id: node_id,
                rectangle: self.tiling_scheme.tile_x_y_to_rectange(
                    node_id.x,
                    node_id.y,
                    node_id.level,
                ),
                neighbours: Default::default(),
                location: location,
            });
            TerrainQuadtreeNode::Leaf(self.leaves.len() - 1)
        }
    }
    pub(crate) fn subdivide(&mut self, node_id: TerrainQuadtreeNode) {
        if let TerrainQuadtreeNode::Internal(index) = node_id {
            let node = &self.internals[index];
            let neighbours = node.neighbours;
            let southwest = node.id.southwest();
            let southeast = node.id.southeast();
            let northwest = node.id.northwest();
            let northeast = node.id.northeast();
            let nw = self.new_node(southwest, Quadrant::Southwest, node_id);
            let ne = self.new_node(southeast, Quadrant::Southeast, node_id);
            let sw = self.new_node(northwest, Quadrant::Northwest, node_id);
            let se = self.new_node(northeast, Quadrant::Northeast, node_id);

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
