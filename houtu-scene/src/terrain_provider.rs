
use std::{collections::HashMap, f64::consts::PI};

use bevy::prelude::{Resource};

use crate::{Ellipsoid, FOUR_GIGABYTES};
#[derive(Debug, Clone)]
pub struct IndicesAndEdges {
    pub indices: Vec<u32>,
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
    pub index_count_without_skirts: Option<u32>,
}
pub struct InnerIndicesAndEdges {
    pub west_indices_south_to_north: Vec<u32>,
    pub south_indices_east_to_west: Vec<u32>,
    pub east_indices_north_to_south: Vec<u32>,
    pub north_indices_west_to_east: Vec<u32>,
}
const heightmap_terrain_quality: f64 = 0.25;
// static mut regular_grid_and_edge_indices_cache: Vec<Vec<IndicesAndEdges>> = Vec::new();
// static mut regular_grid_and_skirt_and_edge_indices_cache: Vec<Vec<IndicesAndEdges>> = Vec::new();
// static mut regular_grid_indices_cache: Vec<Vec<Vec<u32>>> = Vec::new();
#[derive(Resource)]
pub struct IndicesAndEdgesCache {
    pub regular_grid_and_edge_indices_cache: HashMap<String, IndicesAndEdges>,
    pub regular_grid_and_skirt_and_edge_indices_cache: HashMap<String, IndicesAndEdges>,
    pub regular_grid_indices_cache: HashMap<String, Vec<u32>>,
}
impl IndicesAndEdgesCache {
    pub fn new() -> Self {
        Self {
            regular_grid_and_edge_indices_cache: HashMap::new(),
            regular_grid_and_skirt_and_edge_indices_cache: HashMap::new(),
            regular_grid_indices_cache: HashMap::new(),
        }
    }
    fn get_key(width: u32, height: u32) -> String {
        format!("{}-{}", width, height)
    }
    pub fn get_regular_grid_indices(&mut self, width: u32, height: u32) -> Vec<u32> {
        if (width * height) as u64 >= FOUR_GIGABYTES {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self.regular_grid_indices_cache.get(key.as_str()).cloned();

        if value.is_none() {
            let mut new_value: Vec<u32> = vec![0; ((width - 1) * (height - 1) * 6) as usize];
            add_regular_grid_indices(width, height, &mut new_value, 0);
            self.regular_grid_indices_cache
                .insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }
    }
    pub fn get_regular_grid_and_skirt_indices_and_edge_indices(
        &mut self,
        width: u32,
        height: u32,
    ) -> IndicesAndEdges {
        if (width * height) as u64 >= FOUR_GIGABYTES {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self
            .regular_grid_and_skirt_and_edge_indices_cache
            .get(key.as_str())
            .cloned();

        if value.is_none() {
            let grid_vertex_count = width * height;
            let mut grid_index_count = (width - 1) * (height - 1) * 6;
            let edge_vertex_count = width * 2 + height * 2;
            let edge_index_count = 0.max(edge_vertex_count - 4) * 6;
            let _vertex_count = grid_vertex_count + edge_vertex_count;
            let index_count = grid_index_count + edge_index_count;

            let edge_indices = get_edge_indices(width, height);
            let west_indices_south_to_north = edge_indices.west_indices_south_to_north;
            let south_indices_east_to_west = edge_indices.south_indices_east_to_west;
            let east_indices_north_to_south = edge_indices.east_indices_north_to_south;
            let north_indices_west_to_east = edge_indices.north_indices_west_to_east;

            let mut indices: Vec<u32> = vec![0; index_count as usize];
            add_regular_grid_indices(width, height, &mut indices, 0);
            grid_index_count = add_skirt_indices(
                &west_indices_south_to_north,
                &south_indices_east_to_west,
                &east_indices_north_to_south,
                &north_indices_west_to_east,
                grid_vertex_count,
                &mut indices,
                grid_index_count,
            );
            let new_value = IndicesAndEdges {
                indices: indices,
                west_indices_south_to_north: west_indices_south_to_north,
                south_indices_east_to_west: south_indices_east_to_west,
                east_indices_north_to_south: east_indices_north_to_south,
                north_indices_west_to_east: north_indices_west_to_east,
                index_count_without_skirts: Some(grid_index_count),
            };
            self.regular_grid_and_skirt_and_edge_indices_cache
                .insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }
    }
    pub fn get_regular_grid_indices_and_edge_indices(
        &mut self,
        width: u32,
        height: u32,
    ) -> IndicesAndEdges {
        if (width * height) as u64 >= FOUR_GIGABYTES {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self
            .regular_grid_and_edge_indices_cache
            .get(key.as_str())
            .cloned();

        if value.is_none() {
            let indices = self.get_regular_grid_indices(width, height);

            let edge_indices = get_edge_indices(width, height);
            let west_indices_south_to_north = edge_indices.west_indices_south_to_north;
            let south_indices_east_to_west = edge_indices.south_indices_east_to_west;
            let east_indices_north_to_south = edge_indices.east_indices_north_to_south;
            let north_indices_west_to_east = edge_indices.north_indices_west_to_east;

            let new_value = IndicesAndEdges {
                indices: indices,
                west_indices_south_to_north: west_indices_south_to_north,
                south_indices_east_to_west: south_indices_east_to_west,
                east_indices_north_to_south: east_indices_north_to_south,
                north_indices_west_to_east: north_indices_west_to_east,
                index_count_without_skirts: None,
            };
            self.regular_grid_and_edge_indices_cache
                .insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }
    }
    pub fn get_estimated_level_zero_geometric_error_for_a_heightmap(
        ellipsoid: &Ellipsoid,
        tile_image_width: u32,
        number_of_tiles_at_level_zero: u32,
    ) -> f64 {
        return get_estimated_level_zero_geometric_error_for_a_heightmap(
            ellipsoid,
            tile_image_width,
            number_of_tiles_at_level_zero,
        );
    }
}

pub fn get_estimated_level_zero_geometric_error_for_a_heightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    number_of_tiles_at_level_zero: u32,
) -> f64 {
    return (ellipsoid.maximum_radius * 2. * PI * heightmap_terrain_quality)
        / (tile_image_width * number_of_tiles_at_level_zero) as f64;
}

pub fn get_edge_indices(width: u32, height: u32) -> InnerIndicesAndEdges {
    let mut west_indices_south_to_north = vec![0; height as usize];
    let mut south_indices_east_to_west = vec![0; width as usize];
    let mut east_indices_north_to_south = vec![0; height as usize];
    let mut north_indices_west_to_east = vec![0; width as usize];

    for i in 0..width {
        let ii = i as usize;
        north_indices_west_to_east[ii] = i;
        south_indices_east_to_west[ii] = width * height - 1 - i;
    }
    for i in 0..height {
        let ii = i as usize;
        east_indices_north_to_south[ii] = (i + 1) * width - 1;
        west_indices_south_to_north[ii] = (height - i - 1) * width;
    }

    return InnerIndicesAndEdges {
        west_indices_south_to_north,
        south_indices_east_to_west,
        east_indices_north_to_south,
        north_indices_west_to_east,
    };
}
pub fn add_skirt_indices(
    west_indices_south_to_north: &Vec<u32>,
    south_indices_east_to_west: &Vec<u32>,
    east_indices_north_to_south: &Vec<u32>,
    north_indices_west_to_east: &Vec<u32>,
    vertex_count: u32,
    indices: &mut Vec<u32>,
    offset: u32,
) -> u32 {
    let mut offset = offset;
    let mut vertex_index = vertex_count;
    offset = inner_add_skirt_indices(west_indices_south_to_north, vertex_index, indices, offset);
    vertex_index += west_indices_south_to_north.len() as u32;
    offset = inner_add_skirt_indices(south_indices_east_to_west, vertex_index, indices, offset);
    vertex_index += south_indices_east_to_west.len() as u32;
    offset = inner_add_skirt_indices(east_indices_north_to_south, vertex_index, indices, offset);
    vertex_index += east_indices_north_to_south.len() as u32;
    return inner_add_skirt_indices(north_indices_west_to_east, vertex_index, indices, offset);
}
fn inner_add_skirt_indices(
    edge_indices: &Vec<u32>,
    vertex_index: u32,
    indices: &mut Vec<u32>,
    offset: u32,
) -> u32 {
    let mut previous_index = edge_indices[0];
    let length = edge_indices.len();
    let mut uoffset = offset as usize;
    let mut vertex_index = vertex_index;
    for i in 1..length {
        let index = edge_indices[i];

        indices[uoffset] = previous_index;
        uoffset += 1;
        indices[uoffset] = index;
        uoffset += 1;
        indices[uoffset] = vertex_index;
        uoffset += 1;

        indices[uoffset] = vertex_index;
        uoffset += 1;
        indices[uoffset] = index;
        uoffset += 1;
        indices[uoffset] = vertex_index + 1;
        uoffset += 1;

        previous_index = index;
        vertex_index += 1;
    }

    return uoffset as u32;
}
pub fn add_regular_grid_indices(width: u32, height: u32, indices: &mut Vec<u32>, offset: u32) {
    let mut index = 0;
    let mut uoffset = offset as usize;
    for _j in 0..(height - 1) {
        for _i in 0..(width - 1) {
            let upper_left = index;
            let lower_left = upper_left + width;
            let lower_right = lower_left + 1;
            let upper_right = upper_left + 1;

            indices[uoffset] = upper_left;
            uoffset += 1;
            indices[uoffset] = lower_left;
            uoffset += 1;
            indices[uoffset] = upper_right;
            uoffset += 1;
            indices[uoffset] = upper_right;
            uoffset += 1;
            indices[uoffset] = lower_left;
            uoffset += 1;
            indices[uoffset] = lower_right;
            uoffset += 1;

            index += 1;
        }
        index += 1;
    }
}
