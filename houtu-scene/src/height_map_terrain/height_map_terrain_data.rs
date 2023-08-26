use std::{
    any::type_name,
    sync::{Arc, Mutex},
};

use crate::{
    get_estimated_level_zero_geometric_error_for_a_heightmap,
    lerp,
    // get_estimated_level_zero_geometric_error_for_a_heightmap, get_regular_grid_and_skirt_indices_and_edge_indices,
    // get_regular_grid_indices_and_edge_indices,
    CreateVerticeOptions,
    CreateVerticeReturn,
    GeographicTilingScheme,
    HeightmapEncoding,
    HeightmapTerrainStructure,
    IndicesAndEdgesCache,
    Rectangle,
    TerrainEncoding,
    TerrainMesh,
    TileKey,
    TilingScheme,
};

use super::create_vertice;
#[derive(Debug)]
pub struct HeightmapTerrainData {
    pub _buffer: Vec<f32>,
    pub _width: u32,
    pub _height: u32,
    pub _child_tile_mask: i32,
    pub _encoding: HeightmapEncoding,
    pub _structure: HeightmapTerrainStructure,
    pub _created_by_upsampling: bool,
    pub _water_mask: Option<Vec<u8>>,
    pub _skirt_height: Option<f64>,
    pub _mesh: Option<TerrainMesh>,
}

impl HeightmapTerrainData {
    pub fn new(
        buffer: Vec<f32>,
        width: u32,
        height: u32,
        child_tile_mask: Option<i32>,
        encoding: Option<HeightmapEncoding>,
        structure: Option<HeightmapTerrainStructure>,
        created_by_upsampling: Option<bool>,
        water_mask: Option<Vec<u8>>,
        skirt_height: Option<f64>,
        mesh: Option<TerrainMesh>,
    ) -> Self {
        Self {
            _buffer: buffer,
            _width: width,
            _height: height,
            _child_tile_mask: child_tile_mask.unwrap_or(15),
            _encoding: encoding.unwrap_or(HeightmapEncoding::NONE),
            _structure: structure.unwrap_or(HeightmapTerrainStructure::default()),
            _created_by_upsampling: created_by_upsampling.unwrap_or(false),
            _water_mask: water_mask,
            _skirt_height: skirt_height,
            _mesh: mesh,
        }
    }
    pub fn get_mesh(&self) -> Option<&TerrainMesh> {
        return self._mesh.as_ref();
    }
    pub fn has_mesh(&self) -> bool {
        return self._mesh.is_some();
    }
    pub fn can_upsample(&self) -> bool {
        return self._mesh.is_some();
    }
    pub fn is_child_available(&self, thisX: u32, thisY: u32, childX: u32, childY: u32) -> bool {
        let mut bitNumber = 2; // northwest child
        if childX != thisX * 2 {
            bitNumber += 1; // east child
        }
        if childY != thisY * 2 {
            bitNumber -= 2; // south child
        }

        return (self._child_tile_mask & (1 << bitNumber)) != 0;
    }
    pub fn was_created_by_upsampling(&self) -> bool {
        return self._created_by_upsampling;
    }
    pub async fn createMesh<T: TilingScheme>(
        &mut self,
        tiling_scheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggeration_relative_height: Option<f64>,
        indices_and_edges_cache_arc: Arc<Mutex<IndicesAndEdgesCache>>,
    ) {
        let result = self.create_vertice(
            tiling_scheme,
            x,
            y,
            level,
            exaggeration,
            exaggeration_relative_height,
        );

        let mut indices_and_edges_cache = indices_and_edges_cache_arc.lock().unwrap();
        let indices_and_edges;
        if self._skirt_height.unwrap() > 0.0 {
            indices_and_edges = indices_and_edges_cache
                .get_regular_grid_and_skirt_indices_and_edge_indices(self._width, self._height);
        } else {
            indices_and_edges = indices_and_edges_cache
                .get_regular_grid_indices_and_edge_indices(self._width, self._height);
        }

        let vertex_count_without_skirts = 0;
        self._mesh = Some(TerrainMesh::new(
            result.relativeToCenter.unwrap(),
            result.vertices,
            indices_and_edges.indices,
            indices_and_edges.index_count_without_skirts,
            vertex_count_without_skirts,
            Some(result.minimum_height),
            Some(result.maximum_height),
            result.bounding_sphere_3d,
            result.occludee_point_in_scaled_space,
            result.encoding.stride,
            result.oriented_bounding_box,
            result.encoding,
            indices_and_edges.west_indices_south_to_north,
            indices_and_edges.south_indices_east_to_west,
            indices_and_edges.east_indices_north_to_south,
            indices_and_edges.north_indices_west_to_east,
        ));
    }

    pub fn create_vertice<T: TilingScheme>(
        &mut self,
        tiling_scheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggeration_relative_height: Option<f64>,
    ) -> CreateVerticeReturn {
        let tiling_scheme = tiling_scheme;
        let x = x;
        let y = y;
        let level = level;
        let exaggeration = exaggeration.unwrap_or(1.0);
        let exaggeration_relative_height = exaggeration_relative_height.unwrap_or(0.0);

        let ellipsoid = tiling_scheme.get_ellipsoid();
        let native_rectangle = tiling_scheme.tile_x_y_to_native_rectange(x, y, level);
        let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, level);

        // Compute the center of the tile for RTC rendering.
        let center = ellipsoid.cartographic_to_cartesian(&rectangle.center());

        let structure = self._structure;

        let levelZeroMaxError = get_estimated_level_zero_geometric_error_for_a_heightmap(
            &ellipsoid,
            self._width,
            tiling_scheme.get_number_of_x_tiles_at_level(0),
        );
        let thisLevelMaxError = levelZeroMaxError / (1 << level) as f64;
        let skirt_height = (thisLevelMaxError * 4.0).min(1000.0);
        self._skirt_height = Some(skirt_height);
        let result = create_vertice(CreateVerticeOptions {
            heightmap: &mut self._buffer,
            structure: Some(structure),
            includeWebMercatorT: Some(true),
            width: self._width,
            height: self._height,
            native_rectangle: native_rectangle,
            rectangle: Some(rectangle),
            relativeToCenter: Some(center),
            ellipsoid: Some(ellipsoid),
            skirt_height: skirt_height,
            isGeographic: Some(
                type_name::<T>() == "houtu_scene::geographic_tiling_scheme::GeographicTilingScheme",
            ),
            exaggeration: Some(exaggeration),
            exaggeration_relative_height: Some(exaggeration_relative_height),
        });
        return result;
    }
    //上采样 https://zhuanlan.zhihu.com/p/579702765
    pub async fn upsample(
        &self,
        tiling_scheme: &GeographicTilingScheme,
        thisX: u32,
        thisY: u32,
        thisLevel: u32,
        descendantX: u32,
        descendantY: u32,
        descendantLevel: u32,
    ) -> Option<HeightmapTerrainData> {
        if self._mesh.is_none() {
            return None;
        }
        let mesh_data = self._mesh.as_ref().unwrap();

        let width = self._width;
        let height = self._height;
        let structure = self._structure;
        let stride = structure.stride;

        let mut heights: Vec<f32> = vec![0.; (width * height * stride) as usize];

        let buffer = &mesh_data.vertices;
        let encoding = mesh_data.encoding;

        // PERFORMANCE_IDEA: don't recompute these rectangles - the caller already knows them.
        let source_rectangle = tiling_scheme.tile_x_y_to_rectange(thisX, thisY, thisLevel);
        let destination_rectangle =
            tiling_scheme.tile_x_y_to_rectange(descendantX, descendantY, descendantLevel);

        let height_offset = structure.height_offset;
        let height_scale = structure.height_scale;

        let elements_per_height = structure.elements_per_height;
        let element_multiplier = structure.element_multiplier;
        let is_big_endian = structure.is_big_endian;

        let divisor = element_multiplier.pow(elements_per_height - 1);

        for j in 0..height {
            let latitude = lerp(
                destination_rectangle.north,
                destination_rectangle.south,
                (j / (height - 1)) as f64,
            );
            for i in 0..width {
                let longitude = lerp(
                    destination_rectangle.west,
                    destination_rectangle.east,
                    (i / (width - 1)) as f64,
                );
                let mut heightSample = interpolateMeshHeight(
                    &buffer,
                    &encoding,
                    height_offset,
                    height_scale,
                    &source_rectangle,
                    width,
                    height,
                    longitude,
                    latitude,
                );

                // Use conditionals here instead of Math.min and Math.max so that an undefined
                // lowestEncodedHeight or highestEncodedHeight has no effect.
                heightSample = if heightSample < structure.lowestEncodedHeight {
                    structure.lowestEncodedHeight
                } else {
                    heightSample
                };
                heightSample = if heightSample > structure.highestEncodedHeight {
                    structure.highestEncodedHeight
                } else {
                    heightSample
                };

                set_height(
                    &mut heights,
                    elements_per_height,
                    element_multiplier,
                    divisor,
                    stride,
                    is_big_endian,
                    j * width + i,
                    heightSample,
                );
            }
        }
        return Some(HeightmapTerrainData::new(
            heights,
            width,
            height,
            Some(0),
            None,
            Some(self._structure.clone()),
            Some(true),
            None,
            None,
            None,
        ));
    }
}

fn interpolateMeshHeight(
    buffer: &Vec<f32>,
    encoding: &TerrainEncoding,
    height_offset: f64,
    height_scale: f64,
    source_rectangle: &Rectangle,
    width: u32,
    height: u32,
    longitude: f64,
    latitude: f64,
) -> f64 {
    // returns a height encoded according to the structure's height_scale and height_offset.
    let fromWest = ((longitude - source_rectangle.west) * (width - 1) as f64)
        / (source_rectangle.east - source_rectangle.west);
    let fromSouth = ((latitude - source_rectangle.south) * (height - 1) as f64)
        / (source_rectangle.north - source_rectangle.south);

    let mut westInteger = fromWest.round() as u32;
    let mut eastInteger = westInteger + 1;
    if eastInteger >= width {
        eastInteger = width - 1;
        westInteger = width - 2;
    }

    let mut southInteger = fromSouth.round() as u32;
    let mut northInteger = southInteger + 1;
    if northInteger >= height {
        northInteger = height - 1;
        southInteger = height - 2;
    }

    let dx = fromWest - westInteger as f64;
    let dy = fromSouth - southInteger as f64;

    southInteger = height - 1 - southInteger;
    northInteger = height - 1 - northInteger;

    let south_west_height = (encoding
        .decode_height(buffer, (southInteger * width + westInteger) as usize)
        - height_offset)
        / height_scale;
    let south_east_height = (encoding
        .decode_height(buffer, (southInteger * width + eastInteger) as usize)
        - height_offset)
        / height_scale;
    let north_west_height = (encoding
        .decode_height(buffer, (northInteger * width + westInteger) as usize)
        - height_offset)
        / height_scale;
    let north_east_height = (encoding
        .decode_height(buffer, (northInteger * width + eastInteger) as usize)
        - height_offset)
        / height_scale;

    return triangle_interpolate_height(
        dx,
        dy,
        south_west_height,
        south_east_height,
        north_west_height,
        north_east_height,
    );
}

fn triangle_interpolate_height(
    dX: f64,
    dY: f64,
    south_west_height: f64,
    south_east_height: f64,
    north_west_height: f64,
    north_east_height: f64,
) -> f64 {
    // The HeightmapTessellator bisects the quad from southwest to northeast.
    if dY < dX {
        // Lower right triangle
        return (south_west_height
            + dX * (south_east_height - south_west_height)
            + dY * (north_east_height - south_east_height));
    }

    // Upper left triangle
    return (south_west_height
        + dX * (north_east_height - north_west_height)
        + dY * (north_west_height - south_west_height));
}
fn set_height(
    heights: &mut Vec<f32>,
    elements_per_height: u32,
    element_multiplier: u32,
    divisor: u32,
    stride: u32,
    is_big_endian: bool,
    index: u32,
    height: f64,
) {
    let mut height = height as f32;
    let mut divisor = divisor;
    let index = index * stride;
    let mut j = 0;
    if is_big_endian {
        for i in 0..elements_per_height - 1 {
            heights[(index + i) as usize] = (height / divisor as f32).round();
            height -= heights[(index + i) as usize] * divisor as f32;
            divisor /= element_multiplier;
        }
        j = elements_per_height - 2;
    } else {
        for i in (0..elements_per_height - 1).rev() {
            heights[(index + i) as usize] = (height / divisor as f32).round();
            height -= heights[(index + i) as usize] * divisor as f32;
            divisor /= element_multiplier;
        }
        j = 1;
    }
    heights[(index + j) as usize] = height;
}
//   fn get_height(
//     heights,
//     elements_per_height,
//     element_multiplier,
//     stride,
//     is_big_endian,
//     index
//   ) {
//     index *= stride;

//     let height = 0;
//     let i;

//     if is_big_endian {
//       for (i = 0; i < elements_per_height; ++i) {
//         height = height * element_multiplier + heights[index + i];
//       }
//     } else {
//       for (i = elements_per_height - 1; i >= 0; --i) {
//         height = height * element_multiplier + heights[index + i];
//       }
//     }

//     return height;
//   }
