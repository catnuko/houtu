use bevy::utils::HashMap;
use houtu_scene::{GeographicTilingScheme, Rectangle, Tile, TilingScheme};

use super::{
    quadtree_tile::{Direction, Quadrant, QuadtreeTile, TileNode},
    tile_key::TileKey,
};

pub struct QuadtreeTileStorage {
    map: HashMap<TileKey, QuadtreeTile>,
    pub root: Vec<TileKey>,
}
impl Default for QuadtreeTileStorage {
    fn default() -> Self {
        Self::new()
    }
}
impl QuadtreeTileStorage {
    pub fn new() -> Self {
        return Self {
            map: HashMap::new(),
            root: vec![],
        };
    }
    pub fn root_len(&self) -> usize {
        return self.root.len();
    }
    pub fn add(&mut self, mut tile: QuadtreeTile) {
        if let None = tile.parent {
            self.root.push(tile.key.clone())
        }
        self.map.insert(tile.key.clone(), tile);
    }
    pub fn remove(&mut self, key: &TileKey) {
        let value = self.map.remove(key);
        if let Some(v) = value {
            if let Quadrant::Root(index) = v.location {
                self.root.remove(index);
            }
        };
    }
    pub fn get(&self, k: &TileKey) -> Option<&QuadtreeTile> {
        return self.map.get(k);
    }
    pub fn get_mut(&mut self, k: &TileKey) -> Option<&mut QuadtreeTile> {
        return self.map.get_mut(k);
    }
    pub fn get_children_mut(
        &mut self,
        parent_key: &TileKey,
        location: Quadrant,
    ) -> &mut QuadtreeTile {
        let parent = self.get(parent_key).unwrap();
        let southeast = parent.southeast.clone().unwrap();
        let southwest = parent.southwest.clone().unwrap();
        let northeast = parent.northeast.clone().unwrap();
        let northwest = parent.northwest.clone().unwrap();
        return match location {
            Quadrant::Southeast => self.get_mut(&southeast).unwrap(),
            Quadrant::Southwest => self.get_mut(&southwest).unwrap(),
            Quadrant::Northeast => self.get_mut(&northeast).unwrap(),
            Quadrant::Northwest => self.get_mut(&northwest).unwrap(),
            _ => panic!("no children for tile {:?}", parent_key),
        };
    }
    fn make_new_root_tile(
        &self,
        k: &TileKey,
        tiling_scheme: &GeographicTilingScheme,
    ) -> QuadtreeTile {
        let r = tiling_scheme.tile_x_y_to_rectange(k.x, k.y, k.level);
        return QuadtreeTile::new(k.clone(), Quadrant::Root(self.root.len()), None, r);
    }
    pub fn new_root_tile(
        &mut self,
        k: &TileKey,
        tiling_scheme: &GeographicTilingScheme,
    ) -> &mut QuadtreeTile {
        bevy::log::info!("new root tile,key is {:?}", k);
        let tile = self.make_new_root_tile(k, tiling_scheme);
        self.add(tile);
        return self.get_mut(k).unwrap();
    }
    pub fn new_children_tile(
        &mut self,
        parent_key: &TileKey,
        tiling_scheme: &GeographicTilingScheme,
        location: Quadrant,
    ) -> &mut QuadtreeTile {
        let rectangle: Rectangle =
            tiling_scheme.tile_x_y_to_rectange(parent_key.x, parent_key.y, parent_key.level);
        let child_key: TileKey;
        match location {
            Quadrant::Southwest => {
                child_key = parent_key.southwest();
            }
            Quadrant::Southeast => {
                child_key = parent_key.southeast();
            }
            Quadrant::Northwest => {
                child_key = parent_key.northwest();
            }
            Quadrant::Northeast => {
                child_key = parent_key.northeast();
            }
            _ => {
                panic!("error")
            }
        }

        let parent = self.get_mut(parent_key).unwrap();

        let tile = QuadtreeTile::new(
            child_key.clone(),
            location.clone(),
            Some(parent_key.clone()),
            rectangle,
        );
        match location {
            Quadrant::Southwest => {
                parent.southwest = Some(child_key.clone());
            }
            Quadrant::Southeast => {
                parent.southeast = Some(child_key.clone());
            }
            Quadrant::Northwest => {
                parent.northwest = Some(child_key.clone());
            }
            Quadrant::Northeast => {
                parent.northeast = Some(child_key.clone());
            }
            _ => {}
        };
        self.add(tile);
        return self.get_mut(&child_key).unwrap();
    }
    pub fn subdivide(&mut self, parent_key: &TileKey, tiling_scheme: &GeographicTilingScheme) {
        let parent = self.get(parent_key).unwrap();
        if parent.southwest.is_some() {
            return;
        }
        self.new_children_tile(parent_key, tiling_scheme, Quadrant::Southeast);
        self.new_children_tile(parent_key, tiling_scheme, Quadrant::Southwest);
        self.new_children_tile(parent_key, tiling_scheme, Quadrant::Northwest);
        self.new_children_tile(parent_key, tiling_scheme, Quadrant::Northeast);
    }
    pub fn create_level_zero_tiles(&mut self, tiling_scheme: &GeographicTilingScheme) {
        let number_of_level_zero_tiles_x = tiling_scheme.get_number_of_x_tiles_at_level(0);
        let number_of_level_zero_tiles_y = tiling_scheme.get_number_of_y_tiles_at_level(0);
        let mut i = 0;
        for y in 0..number_of_level_zero_tiles_y {
            for x in 0..number_of_level_zero_tiles_x {
                let r = tiling_scheme.tile_x_y_to_rectange(x, y, 0);
                self.new_root_tile(
                    &TileKey {
                        x: x,
                        y: y,
                        level: 0,
                    },
                    tiling_scheme,
                );
                i += 1;
            }
        }
    }
    pub fn get_root_tile(&self) -> Vec<&QuadtreeTile> {
        let mut res: Vec<&QuadtreeTile> = vec![];
        for i in self.root.iter() {
            let v = self.get(i).unwrap();
            res.push(v);
        }
        return res;
    }
}
