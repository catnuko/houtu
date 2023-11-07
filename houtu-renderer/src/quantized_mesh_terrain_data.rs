use bevy::math::DVec3;
use houtu_scene::{BoundingSphere, OrientedBoundingBox, TilingScheme};
use quantized_mesh_decoder::{from_reader, Indices};

use crate::quadtree::{terrain_provider::TerrainProvider, tile_key::TileKey};

pub struct QuantizedMeshTerrainData {
    quantized_vertices: Vec<u16>,
    encoded_normals: Option<Vec<u8>>,
    indices: Indices,
    minimum_height: f32,
    maximum_height: f32,
    bounding_sphere: BoundingSphere,
    oriented_bounding_box: OrientedBoundingBox,
    horizon_occlusion_point: DVec3,
    west_indices: Indices,
    south_indices: Indices,
    east_indices: Indices,
    north_indices: Indices,
    west_skirt_height: f64,
    south_skirt_height: f64,
    east_skirt_height: f64,
    north_skirt_height: f64,
    child_tile_mask: u32,
    created_by_upsampling: bool,
    water_mask: Option<Vec<u8>>,
}
impl QuantizedMeshTerrainData {
    fn from_data(
        value: quantized_mesh_decoder::QuantizedMeshTerrainData,
        tile_key: TileKey,
        terrain_provider: &mut Box<dyn TerrainProvider>,
    ) -> Self {
        let h = &value.header;
        let bounding_sphere = BoundingSphere::new(
            DVec3 {
                x: h.bounding_sphere_center_x,
                y: h.bounding_sphere_center_y,
                z: h.bounding_sphere_center_z,
            },
            h.bounding_sphere_radius,
        );
        let skirt_height = terrain_provider.get_level_maximum_geometric_error(tile_key.level) * 5.0;
        let rectangle = terrain_provider.get_tiling_scheme().tile_x_y_to_rectange(
            tile_key.x,
            tile_key.y,
            tile_key.level,
        );
        let oriented_bounding_box = OrientedBoundingBox::from_rectangle(
            &rectangle,
            Some(h.minimum_height as f64),
            Some(h.maximum_height as f64),
            Some(&terrain_provider.get_tiling_scheme().ellipsoid),
        );
        Self {
            quantized_vertices: value.vertex_data,
            encoded_normals: value.extension.vertex_normals,
            indices: value.triangle_indices,
            minimum_height: value.header.minimum_height,
            maximum_height: value.header.maximum_height,
            bounding_sphere: bounding_sphere,
            oriented_bounding_box: oriented_bounding_box,
            horizon_occlusion_point: DVec3 {
                x: h.horizon_occlusion_point_x,
                y: h.horizon_occlusion_point_y,
                z: h.horizon_occlusion_point_z,
            },
            west_indices: value.west_indices,
            south_indices: value.south_indices,
            east_indices: value.east_indices,
            north_indices: value.north_indices,
            west_skirt_height: skirt_height,
            south_skirt_height: skirt_height,
            east_skirt_height: skirt_height,
            north_skirt_height: skirt_height,
            child_tile_mask: terrain_provider
                .get_availability()
                .unwrap()
                .compute_child_mask_for_tile(tile_key.level, tile_key.x, tile_key.y),
            created_by_upsampling: false,
            water_mask: value.extension.water_mask,
        }
    }
}
