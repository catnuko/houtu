use bevy::{
    math::DVec3,
    prelude::shape::Quad,
    utils::{petgraph::adj::NodeIndex, HashMap},
};
use houtu_scene::{Cartographic, GeographicTilingScheme, Rectangle, TilingScheme};

use super::{quadtree_tile::Quadrant, tile_key::TileKey};

pub struct TileAvailability {
    pub tiling_scheme: GeographicTilingScheme,
    pub maximum_level: u32,
    pub storage: QuadtreeNodeStorage,
}
impl TileAvailability {
    pub fn new(max_level: u32) -> Self {
        return Self {
            tiling_scheme: GeographicTilingScheme::default(),
            maximum_level: max_level,
            storage: QuadtreeNodeStorage::new(),
        };
    }
    pub fn compute_child_mask_for_tile(&mut self, level: u32, x: u32, y: u32) -> u32 {
        let child_level = level + 1;
        if child_level >= self.maximum_level {
            return 0;
        }
        let mut mask = 0;
        mask |= if self.is_tile_available(child_level, 2 * x, 2 * y + 1) {
            1
        } else {
            0
        };
        mask |= if self.is_tile_available(child_level, 2 * x + 1, 2 * y + 1) {
            2
        } else {
            0
        };
        mask |= if self.is_tile_available(child_level, 2 * x, 2 * y) {
            4
        } else {
            0
        };
        mask |= if self.is_tile_available(child_level, 2 * x + 1, 2 * y) {
            8
        } else {
            0
        };
        return mask;
    }
    pub fn is_tile_available(&mut self, level: u32, x: u32, y: u32) -> bool {
        let retangle = self.tiling_scheme.tile_x_y_to_rectange(x, y, level);
        let center = retangle.center();
        return self.compute_maximum_level_at_position(&center) >= level as i32;
    }
    pub fn compute_maximum_level_at_position(&mut self, position: &Cartographic) -> i32 {
        let mut node = None;
        for root_node_key in self.storage.root_nodes.iter() {
            let root_node = self.storage.map.get(root_node_key).unwrap();
            if rectangle_contains_position(RectangleLike::Rectangle(&root_node.extent), position) {
                node = Some(root_node);
                break;
            }
        }
        if node.is_none() {
            return -1;
        }
        let node = node.unwrap();
        return self
            .storage
            .find_max_level_from_node(None, node.key, position) as i32;
    }

    pub fn add_available_tile_range(
        &mut self,
        level: u32,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
    ) {
        if level == 0 {
            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    if !self.storage.find_node(level, x, y) {
                        let key = TileKey { x, y, level };
                        self.storage.make_node(key, None);
                        self.storage.root_nodes.push(key);
                    }
                }
            }
        }
        let rectangle_scratch = self
            .tiling_scheme
            .tile_x_y_to_rectange(start_x, start_y, level);
        let rectangle_scratch2 = self.tiling_scheme.tile_x_y_to_rectange(end_x, end_y, level);
        let rectangle_with_level = RectangleWithLevel {
            level,
            west: rectangle_scratch.west,
            north: rectangle_scratch.north,
            east: rectangle_scratch2.east,
            south: rectangle_scratch2.south,
        };
        for i in 0..self.storage.root_nodes.len() {
            let root_node_key = self.storage.root_nodes[i];
            let root_ndoe = self.storage.map.get(&root_node_key).unwrap();
            if rectangles_overlap(
                RectangleLike::Rectangle(&root_ndoe.extent),
                RectangleLike::RectangleWithLevel(&rectangle_with_level),
            ) {
                self.storage.put_rectangle_in_quadtree(
                    self.maximum_level,
                    &root_node_key,
                    &rectangle_with_level,
                );
            }
        }
    }
}
pub enum RectangleLike<'a> {
    Rectangle(&'a Rectangle),
    RectangleWithLevel(&'a RectangleWithLevel),
}
fn rectangles_overlap(rectangle1: RectangleLike, rectangle2: RectangleLike) -> bool {
    let r1 = match rectangle1 {
        RectangleLike::Rectangle(v) => (v.west, v.south, v.east, v.north),
        RectangleLike::RectangleWithLevel(v) => (v.west, v.south, v.east, v.north),
    };
    let r2 = match rectangle2 {
        RectangleLike::Rectangle(v) => (v.west, v.south, v.east, v.north),
        RectangleLike::RectangleWithLevel(v) => (v.west, v.south, v.east, v.north),
    };
    let west = r1.0.max(r2.0);
    let south = r1.1.max(r2.1);
    let east = r1.2.min(r2.2);
    let north = r1.3.min(r2.3);
    return south < north && west < east;
}
fn rectangle_fully_contains_rectangle(
    potential_container: RectangleLike,
    rectangle_to_test: RectangleLike,
) -> bool {
    let r1 = match potential_container {
        RectangleLike::Rectangle(v) => (v.west, v.south, v.east, v.north),
        RectangleLike::RectangleWithLevel(v) => (v.west, v.south, v.east, v.north),
    };
    let r2 = match rectangle_to_test {
        RectangleLike::Rectangle(v) => (v.west, v.south, v.east, v.north),
        RectangleLike::RectangleWithLevel(v) => (v.west, v.south, v.east, v.north),
    };
    return (r2.0 >= r1.0 && r2.2 <= r1.2 && r2.1 >= r1.1 && r2.3 <= r1.3);
}
fn rectangle_contains_position(
    potential_container: RectangleLike,
    position: &Cartographic,
) -> bool {
    let r1 = match potential_container {
        RectangleLike::Rectangle(v) => (v.west, v.south, v.east, v.north),
        RectangleLike::RectangleWithLevel(v) => (v.west, v.south, v.east, v.north),
    };
    return (position.longitude >= r1.0
        && position.longitude <= r1.2
        && position.latitude >= r1.1
        && position.latitude <= r1.3);
}
fn subtract_rectangle(
    rectangle_list: &Vec<Rectangle>,
    rectangle_to_subtract: &Rectangle,
) -> Vec<Rectangle> {
    let mut result = vec![];
    for i in 0..rectangle_list.len() {
        let rectangle = rectangle_list[i];
        if !rectangles_overlap(
            RectangleLike::Rectangle(&rectangle),
            RectangleLike::Rectangle(rectangle_to_subtract),
        ) {
            // Disjoint rectangles.  Original rectangle is unmodified.

            result.push(rectangle.clone())
        } else {
            // rectangleToSubtract partially or completely overlaps rectangle.
            if rectangle.west < rectangle_to_subtract.west {
                result.push(Rectangle::new(
                    rectangle.west,
                    rectangle.south,
                    rectangle_to_subtract.west,
                    rectangle.north,
                ));
            }
            if rectangle.east > rectangle_to_subtract.east {
                result.push(Rectangle::new(
                    rectangle_to_subtract.east,
                    rectangle.south,
                    rectangle.east,
                    rectangle.north,
                ));
            }
            if rectangle.south < rectangle_to_subtract.south {
                result.push(Rectangle::new(
                    rectangle_to_subtract.west.max(rectangle.west),
                    rectangle.south,
                    rectangle_to_subtract.east.min(rectangle.east),
                    rectangle_to_subtract.south,
                ));
            }
            if rectangle.north > rectangle_to_subtract.north {
                result.push(Rectangle::new(
                    rectangle_to_subtract.west.max(rectangle.west),
                    rectangle_to_subtract.north,
                    rectangle_to_subtract.east.min(rectangle.east),
                    rectangle.north,
                ));
            }
        }
    }
    return result;
}
#[derive(Clone)]
pub struct RectangleWithLevel {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
    pub level: u32,
}
pub struct QuadtreeNodeStorage {
    pub tiling_scheme: GeographicTilingScheme,
    map: HashMap<TileKey, QuadtreeNode>,
    pub root_nodes: Vec<TileKey>,
}
impl QuadtreeNodeStorage {
    pub fn new() -> Self {
        return Self {
            tiling_scheme: GeographicTilingScheme::default(),
            map: HashMap::new(),
            root_nodes: vec![],
        };
    }
    fn put_rectangle_in_quadtree(
        &mut self,
        max_depth: u32,
        node_key: &TileKey,
        rectangle: &RectangleWithLevel,
    ) {
        let mut node_key = node_key.clone();
        while node_key.level < max_depth {
            let nw = {
                let res = self.get_child_mut(&node_key, Quadrant::Northwest).unwrap();
                (res.extent.clone(), res.key)
            };
            let ne = {
                let res = self.get_child_mut(&node_key, Quadrant::Northeast).unwrap();
                (res.extent.clone(), res.key)
            };
            let sw = {
                let res = self.get_child_mut(&node_key, Quadrant::Southwest).unwrap();
                (res.extent.clone(), res.key)
            };
            let se = {
                let res = self.get_child_mut(&node_key, Quadrant::Southeast).unwrap();
                (res.extent.clone(), res.key)
            };
            if rectangle_fully_contains_rectangle(
                RectangleLike::Rectangle(&nw.0),
                RectangleLike::RectangleWithLevel(&rectangle),
            ) {
                node_key = nw.1;
            } else if rectangle_fully_contains_rectangle(
                RectangleLike::Rectangle(&ne.0),
                RectangleLike::RectangleWithLevel(&rectangle),
            ) {
                node_key = ne.1;
            } else if rectangle_fully_contains_rectangle(
                RectangleLike::Rectangle(&sw.0),
                RectangleLike::RectangleWithLevel(&rectangle),
            ) {
                node_key = sw.1;
            } else if rectangle_fully_contains_rectangle(
                RectangleLike::Rectangle(&se.0),
                RectangleLike::RectangleWithLevel(&rectangle),
            ) {
                node_key = se.1;
            } else {
                break;
            }
        }
        let node = self.map.get_mut(&node_key).unwrap();
        if node.rectangles.len() == 0
            || node.rectangles[node.rectangles.len() - 1].level <= rectangle.level
        {
            node.rectangles.push(rectangle.clone())
        } else {
            match node
                .rectangles
                .binary_search_by(|probe| probe.level.cmp(&rectangle.level))
            {
                Err(err_index) => {
                    let index = !err_index;
                    node.rectangles.splice(index..index, [rectangle.clone()]);
                }
                Ok(index) => {
                    node.rectangles.splice(index..index, [rectangle.clone()]);
                }
            }
        }
    }
    fn find_node(&mut self, level: u32, x: u32, y: u32) -> bool {
        for i in 0..self.root_nodes.len() {
            let node = self.root_nodes[i];
            if node.x == x && node.y == y && node.level == level {
                return true;
            }
        }
        return false;
    }
    fn find_max_level_from_node(
        &mut self,
        stop_node_key: Option<TileKey>,
        node_key: TileKey,
        position: &Cartographic,
    ) -> u32 {
        let mut max_level: u32 = 0;
        let mut found = false;
        let mut node_key = node_key.clone();
        //node一定存在
        while !found {
            let node = self.map.get(&node_key).unwrap();
            let nw = {
                let res = node.nw.as_ref().and_then(|x| {
                    let child_node = self.map.get(x).unwrap();
                    Some(rectangle_contains_position(
                        RectangleLike::Rectangle(&child_node.extent),
                        position,
                    ))
                });
                match res.is_some() && res.unwrap() {
                    true => 1,
                    false => 0,
                }
            };
            let ne = {
                let res = node.ne.as_ref().and_then(|x| {
                    let child_node = self.map.get(x).unwrap();
                    Some(rectangle_contains_position(
                        RectangleLike::Rectangle(&child_node.extent),
                        position,
                    ))
                });
                match res.is_some() && res.unwrap() {
                    true => 1,
                    false => 0,
                }
            };
            let sw = {
                let res = node.sw.as_ref().and_then(|x| {
                    let child_node = self.map.get(x).unwrap();
                    Some(rectangle_contains_position(
                        RectangleLike::Rectangle(&child_node.extent),
                        position,
                    ))
                });
                match res.is_some() && res.unwrap() {
                    true => 1,
                    false => 0,
                }
            };
            let se = {
                let res = node.se.as_ref().and_then(|x| {
                    let child_node = self.map.get(x).unwrap();
                    Some(rectangle_contains_position(
                        RectangleLike::Rectangle(&child_node.extent),
                        position,
                    ))
                });
                match res.is_some() && res.unwrap() {
                    true => 1,
                    false => 0,
                }
            };
            if nw + ne + sw + se > 1 {
                if nw == 1 {
                    max_level = max_level.max(self.find_max_level_from_node(
                        Some(node_key),
                        node_key.northwest(),
                        position,
                    ));
                }
                if ne == 1 {
                    max_level = max_level.max(self.find_max_level_from_node(
                        Some(node_key),
                        node_key.northeast(),
                        position,
                    ));
                }
                if sw == 1 {
                    max_level = max_level.max(self.find_max_level_from_node(
                        Some(node_key),
                        node_key.southwest(),
                        position,
                    ));
                }
                if se == 1 {
                    max_level = max_level.max(self.find_max_level_from_node(
                        Some(node_key),
                        node_key.southeast(),
                        position,
                    ));
                }
                break;
            } else if nw == 1 {
                node_key = node_key.northwest();
            } else if ne == 1 {
                node_key = node_key.northeast();
            } else if sw == 1 {
                node_key = node_key.southwest();
            } else if se == 1 {
                node_key = node_key.southeast();
            } else {
                found = true;
            }
        }
        let mut new_node_key = Some(node_key.clone());
        while new_node_key != stop_node_key {
            let new_node = self.map.get(new_node_key.as_ref().unwrap()).unwrap();
            let mut i = new_node.rectangles.len() as i32 - 1;
            while i >= 0 && new_node.rectangles[i as usize].level > max_level {
                let rectangle = &new_node.rectangles[i as usize];
                if rectangle_contains_position(
                    RectangleLike::RectangleWithLevel(rectangle),
                    position,
                ) {
                    max_level = rectangle.level;
                }
                i -= 1;
            }
            new_node_key = new_node.parent.clone();
        }
        return max_level;
    }
    pub fn get_child_mut(
        &mut self,
        key: &TileKey,
        location: Quadrant,
    ) -> Option<&mut QuadtreeNode> {
        let node = self.map.get(key);
        if node.is_none() {
            return None;
        }
        let node = node.unwrap();
        let node_key = node.key.clone();
        match location {
            Quadrant::Northeast => {
                if node.ne.is_some() {
                    let child_key = node.ne.as_ref().unwrap().clone();
                    return self.map.get_mut(&child_key);
                } else {
                    return Some(self.add_ne(&node_key));
                }
            }

            Quadrant::Northwest => {
                if node.nw.is_some() {
                    let child_key = node.nw.as_ref().unwrap().clone();
                    return self.map.get_mut(&child_key);
                } else {
                    return Some(self.add_nw(&node_key));
                }
            }
            Quadrant::Southeast => {
                if node.se.is_some() {
                    let child_key = node.se.as_ref().unwrap().clone();
                    return self.map.get_mut(&child_key);
                } else {
                    return Some(self.add_se(&node_key));
                }
            }
            Quadrant::Southwest => {
                if node.sw.is_some() {
                    let child_key = node.sw.as_ref().unwrap().clone();
                    return self.map.get_mut(&child_key);
                } else {
                    return Some(self.add_sw(&node_key));
                }
            }
            _ => return None,
        };
    }
    fn make_node(&mut self, key: TileKey, parent: Option<TileKey>) -> &mut QuadtreeNode {
        let extent = self
            .tiling_scheme
            .tile_x_y_to_rectange(key.x, key.y, key.level);
        self.map.insert(key, QuadtreeNode::new(key, parent, extent));
        return self.map.get_mut(&key).unwrap();
    }
    fn add_nw(&mut self, node_key: &TileKey) -> &mut QuadtreeNode {
        let key = node_key.northwest();
        let node = self.map.get_mut(node_key).unwrap();
        node.nw = Some(key);
        return self.make_node(key, Some(node_key.clone()));
    }
    fn add_ne(&mut self, node_key: &TileKey) -> &mut QuadtreeNode {
        let key = node_key.northeast();
        let node = self.map.get_mut(node_key).unwrap();
        node.ne = Some(key);
        return self.make_node(key, Some(node_key.clone()));
    }
    fn add_sw(&mut self, node_key: &TileKey) -> &mut QuadtreeNode {
        let key = node_key.southwest();
        let node = self.map.get_mut(node_key).unwrap();
        node.sw = Some(key);
        return self.make_node(key, Some(node_key.clone()));
    }
    fn add_se(&mut self, node_key: &TileKey) -> &mut QuadtreeNode {
        let key = node_key.southeast();
        let node = self.map.get_mut(node_key).unwrap();
        node.se = Some(key);
        return self.make_node(key, Some(node_key.clone()));
    }
}
pub struct QuadtreeNode {
    key: TileKey,
    parent: Option<TileKey>,
    sw: Option<TileKey>,
    se: Option<TileKey>,
    nw: Option<TileKey>,
    ne: Option<TileKey>,
    extent: Rectangle,
    rectangles: Vec<RectangleWithLevel>,
}
impl QuadtreeNode {
    pub fn new(key: TileKey, parent: Option<TileKey>, extent: Rectangle) -> Self {
        QuadtreeNode {
            key,
            parent: parent,
            sw: None,
            se: None,
            nw: None,
            ne: None,
            extent,
            rectangles: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    fn create_availability(max_level: u32) -> TileAvailability {
        let mut availability = TileAvailability::new(max_level);
        availability.add_available_tile_range(
            0,
            0,
            0,
            availability.tiling_scheme.get_number_of_x_tiles_at_level(0),
            availability.tiling_scheme.get_number_of_y_tiles_at_level(0),
        );
        return availability;
    }
    #[test]
    fn return_0_if_there_area_no_rectangles() {
        let mut availability = create_availability(15);
        assert!(
            availability
                .compute_maximum_level_at_position(&Cartographic::from_degrees(25.0, 88.0, 0.0))
                == 0
        );
    }
    #[test]
    fn return_the_higher_level_when_on_a_boundary_as_level_0() {
        let mut availability = create_availability(15);
        availability.add_available_tile_range(0, 0, 0, 0, 0);
        availability.add_available_tile_range(1, 1, 0, 1, 0);
        assert!(
            availability
                .compute_maximum_level_at_position(&Cartographic::from_radians(0.0, 0.0, 0.0))
                == 1
        );
        let mut availability = create_availability(15);
        availability.add_available_tile_range(1, 1, 0, 1, 0);
        availability.add_available_tile_range(0, 0, 0, 0, 0);
        assert!(
            availability
                .compute_maximum_level_at_position(&Cartographic::from_radians(0.0, 0.0, 0.0))
                == 1
        );
    }
    #[test]
    fn return_the_higher_level_when_on_a_boundary_at_level_1() {
        let mut availability = create_availability(15);
        availability.add_available_tile_range(0, 0, 0, 1, 0);
        availability.add_available_tile_range(1, 1, 0, 1, 1);
        assert!(
            availability.compute_maximum_level_at_position(&Cartographic::from_radians(
                -PI / 2.0,
                0.0,
                0.0
            )) == 1
        );
    }
    // #[test]
    // fn return_0_if_there_are_no_rectangles() {
    //     let mut availability = create_availability(15);
    // }
}
