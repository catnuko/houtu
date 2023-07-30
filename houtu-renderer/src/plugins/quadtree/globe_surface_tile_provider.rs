use bevy::math::DVec3;
use houtu_scene::{
    BoundingVolume, Cartesian3, Ellipsoid, EllipsoidalOccluder, GeographicProjection,
    GeographicTilingScheme, Intersect, Rectangle, TileBoundingRegion,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    ellipsoid_terrain_provider::EllipsoidTerrainProvider, quadtree_tile::QuadtreeTile,
    quadtree_tile_storage::QuadtreeTileStorage, terrain_provider::TerrainProvider,
    tile_key::TileKey,
};

pub struct GlobeSurfaceTileProvider {
    terrain_provider: Box<dyn TerrainProvider>,
}
#[derive(Debug, PartialEq)]
pub enum TileVisibility {
    NONE = -1,
    PARTIAL = 0,
    FULL = 1,
}
impl GlobeSurfaceTileProvider {
    pub fn new() -> Self {
        Self {
            terrain_provider: Box::new(EllipsoidTerrainProvider::new()),
        }
    }
    pub fn get_tiling_scheme(&self) -> &GeographicTilingScheme {
        return self.terrain_provider.get_tiling_scheme();
    }
    pub fn compute_tile_load_priority(&mut self) -> f64 {
        0.
    }
    pub fn computeTileVisibility(
        &mut self,
        storage: &mut QuadtreeTileStorage,
        ellipsoidal_occluder: &EllipsoidalOccluder,
        camera: &mut GlobeCamera,
        tile: &mut QuadtreeTile,
    ) -> TileVisibility {
        computeDistanceToTile(storage, ellipsoidal_occluder, camera, tile);
        let surface_tile = &mut tile.data;
        let tile_bounding_region = surface_tile
            .tile_bounding_region
            .as_ref()
            .expect("globe_surface_tile.tileBoundingRegion不存在");
        if let None = surface_tile.bounding_volume_source_tile {
            return TileVisibility::PARTIAL;
        }
        let obb = tile_bounding_region.get_bounding_volume();
        let mut bounding_volume: Option<Box<&dyn BoundingVolume>> = if let Some(v) = obb {
            Some(Box::new(v))
        } else {
            if let Some(t) = tile_bounding_region.get_bounding_sphere() {
                Some(Box::new(t))
            } else {
                None
            }
        };

        surface_tile.clipped_by_boundaries = false;
        if let None = bounding_volume {
            return TileVisibility::PARTIAL;
        }
        let bounding_volume = bounding_volume.unwrap();
        let mut visibility = TileVisibility::NONE;
        let intersection = camera
            .get_culling_volume()
            .computeVisibility(&bounding_volume);

        if intersection == Intersect::OUTSIDE {
            visibility = TileVisibility::NONE;
        } else if intersection == Intersect::INTERSECTING {
            visibility = TileVisibility::PARTIAL;
        } else if intersection == Intersect::INSIDE {
            visibility = TileVisibility::FULL;
        }

        if visibility == TileVisibility::NONE {
            return visibility;
        }

        let occludee_point_in_scaled_space = surface_tile.occludee_point_in_scaled_space;
        if occludee_point_in_scaled_space.is_none() {
            return visibility;
        }
        let occludee_point_in_scaled_space = occludee_point_in_scaled_space.unwrap();
        if (ellipsoidal_occluder.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
            &occludee_point_in_scaled_space,
            Some(tile_bounding_region.minimum_height),
        )) {
            return visibility;
        }

        return TileVisibility::NONE;
    }
    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        return self
            .terrain_provider
            .get_level_maximum_geometric_error(level);
    }
    pub fn can_render_without_losing_detail(&mut self, tile: &mut QuadtreeTile) -> bool {
        true
    }
    pub fn can_refine(&self, tile: &mut QuadtreeTile) -> bool {
        if tile.data.terrain_data.is_some() {
            return true;
        }
        let key = &tile.key;
        let new_key = TileKey::new(key.x * 2, key.y * 2, key.level + 1);
        let child_available = self.terrain_provider.get_tile_data_available(&new_key);
        return child_available != None;
    }
}
fn computeDistanceToTile(
    storage: &mut QuadtreeTileStorage,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    camera: &mut GlobeCamera,
    tile: &mut QuadtreeTile,
) -> f64 {
    update_tile_bounding_region(storage, ellipsoidal_occluder, tile);
    let surface_tile = &mut tile.data;
    let bounding_volume_surface_tile = surface_tile
        .bounding_volume_source_tile
        .and_then(|x| storage.get_mut(&x));
    if let None = bounding_volume_surface_tile {
        return 9999999999.0;
    }
    let tile_bounding_region = surface_tile
        .tile_bounding_region
        .as_mut()
        .expect("globe_surface_tile.tileBoundingRegion不存在");
    let min = tile_bounding_region.minimum_height;
    let max = tile_bounding_region.maximum_height;
    if surface_tile.bounding_volume_source_tile != Some(tile.key) {
        let p = camera.get_position_cartographic();
        let distance_to_min = (p.height - min).abs();
        let distance_to_max = (p.height - max).abs();
        if distance_to_min > distance_to_max {
            tile_bounding_region.minimum_height = min;
            tile_bounding_region.maximum_height = min;
        } else {
            tile_bounding_region.minimum_height = max;
            tile_bounding_region.maximum_height = max;
        }
    }
    tile_bounding_region.minimum_height = min;
    tile_bounding_region.maximum_height = min;
    let distance = tile_bounding_region.distanceToCameraRegion(
        &camera.get_position_wc(),
        &camera.get_position_cartographic(),
        &GeographicProjection::WGS84,
    );
    tile.distance = distance;
    return distance;
}

pub fn update_tile_bounding_region(
    storage: &mut QuadtreeTileStorage,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    tile: &mut QuadtreeTile,
) {
    let surface_tile = &mut tile.data;
    if let None = surface_tile.tile_bounding_region {
        surface_tile.tile_bounding_region = Some(TileBoundingRegion::new(
            &tile.rectangle,
            Some(0.0),
            Some(0.0),
            Some(&Ellipsoid::WGS84),
            Some(false),
        ));
    };
    let tile_bounding_region = surface_tile.tile_bounding_region.as_mut().unwrap();
    let old_minimum_height = tile_bounding_region.minimum_height;
    let old_maximum_height = tile_bounding_region.maximum_height;
    let mut has_bounding_volumes_from_mesh = false;
    let mut source_tile = Some(tile);

    let mesh = surface_tile.mesh.as_ref();
    if let Some(_mesh) = mesh {
        if let (Some(min), Some(max)) = (_mesh.minimum_height, _mesh.maximum_height) {
            tile_bounding_region.minimum_height = min;
            tile_bounding_region.maximum_height = max;
            has_bounding_volumes_from_mesh = true
        }
    } else {
        tile_bounding_region.minimum_height = -1.;
        tile_bounding_region.maximum_height = -1.;
        let mut ancestor_tile = tile.parent.and_then(|x| storage.get_mut(&x));
        while let Some(in_ancestor_tile) = ancestor_tile {
            let ancestor_surface_tile = in_ancestor_tile.data;
            if let Some(ancestor_surface_tile_mesh) = ancestor_surface_tile.mesh {
                if let (Some(min), Some(max)) = (
                    ancestor_surface_tile_mesh.minimum_height,
                    ancestor_surface_tile_mesh.maximum_height,
                ) {
                    tile_bounding_region.minimum_height = min;
                    tile_bounding_region.maximum_height = max;
                    break;
                }
            }
            ancestor_tile = in_ancestor_tile.parent.and_then(|x| storage.get_mut(&x));
        }
        source_tile = ancestor_tile;
    }
    if let Some(in_source_tile) = source_tile {
        if has_bounding_volumes_from_mesh {
            if !surface_tile.bounding_volume_is_from_mesh {
                tile_bounding_region.oriented_bounding_box = Some(
                    surface_tile
                        .mesh
                        .as_ref()
                        .unwrap()
                        .oriented_bounding_box
                        .clone(),
                );
                tile_bounding_region.boundingSphere = Some(
                    surface_tile
                        .mesh
                        .as_ref()
                        .unwrap()
                        .bounding_sphere_3d
                        .clone(),
                );

                surface_tile.occludee_point_in_scaled_space = surface_tile
                    .mesh
                    .as_ref()
                    .unwrap()
                    .occludee_point_in_scaled_space
                    .clone();
                if let None = surface_tile.occludee_point_in_scaled_space {
                    surface_tile.occludee_point_in_scaled_space = compute_occludee_point(
                        ellipsoidal_occluder,
                        &tile_bounding_region.oriented_bounding_box.unwrap().center,
                        &tile.rectangle,
                        tile_bounding_region.minimum_height,
                        tile_bounding_region.maximum_height,
                    )
                }
            }
        } else {
            let needs_bounds = if let Some(_) = tile_bounding_region.oriented_bounding_box {
                true
            } else {
                false
            } || if let Some(_) = tile_bounding_region.boundingSphere {
                true
            } else {
                false
            };
            let height_changed = tile_bounding_region.minimum_height != old_minimum_height
                || tile_bounding_region.maximum_height != old_maximum_height;
            if height_changed || needs_bounds {
                tile_bounding_region.computeBoundingVolumes(&ellipsoidal_occluder.ellipsoid);
                surface_tile.occludee_point_in_scaled_space = compute_occludee_point(
                    ellipsoidal_occluder,
                    &tile_bounding_region.oriented_bounding_box.unwrap().center,
                    &tile.rectangle,
                    tile_bounding_region.minimum_height,
                    tile_bounding_region.maximum_height,
                )
            }
        }
        surface_tile.bounding_volume_source_tile = Some(source_tile.unwrap().key);
        surface_tile.bounding_volume_is_from_mesh = has_bounding_volumes_from_mesh;
    } else {
        surface_tile.bounding_volume_source_tile = None;
        surface_tile.bounding_volume_is_from_mesh = false;
    }
}

fn compute_occludee_point(
    ellipsoidal_occluder: &EllipsoidalOccluder,
    center: &DVec3,
    rectangle: &Rectangle,
    minimum_height: f64,
    maximum_height: f64,
) -> Option<DVec3> {
    let mut corner_positions = vec![DVec3::ZERO, DVec3::ZERO, DVec3::ZERO, DVec3::ZERO];
    let ellipsoid = ellipsoidal_occluder.ellipsoid;
    corner_positions[0] = DVec3::from_radians(
        rectangle.west,
        rectangle.south,
        Some(maximum_height),
        Some(ellipsoid.radii_squared),
    );
    corner_positions[1] = DVec3::from_radians(
        rectangle.east,
        rectangle.south,
        Some(maximum_height),
        Some(ellipsoid.radii_squared),
    );
    corner_positions[2] = DVec3::from_radians(
        rectangle.west,
        rectangle.north,
        Some(maximum_height),
        Some(ellipsoid.radii_squared),
    );
    corner_positions[3] = DVec3::from_radians(
        rectangle.east,
        rectangle.north,
        Some(maximum_height),
        Some(ellipsoid.radii_squared),
    );

    return ellipsoidal_occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
        center,
        &corner_positions,
        minimum_height,
    );
}
