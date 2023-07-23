use bevy::{math::DVec3, prelude::*};
use houtu_scene::{
    BoundingVolume, Cartesian3, Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder,
    GeographicTilingScheme, Intersect, Projection, Rectangle, TileBoundingRegion, TilingScheme,
    WebMercatorTilingScheme, EPSILON5,
};
use std::{collections::HashMap, f64::consts::PI};

use crate::plugins::quadtree::{Quadtree, QuadtreeTile, QuadtreeTileValue};

use super::{
    datasource::DataSource, layer_id::LayerId, tile_layer_state::TileLayerState, tile_z::Tile,
};

#[derive(Clone, Debug, Resource)]
pub struct TileLayer<D: DataSource, T: TilingScheme> {
    pub quadtree: Quadtree<QuadtreeTile>,
    pub tiles: HashMap<String, Entity>,
    pub tiling_scheme: T,
    pub state: TileLayerState,
    pub cartographicLimitRectangle: Rectangle,
    pub _occluders: EllipsoidalOccluder,
    pub datasource: D,
    pub _lastSelectionFrameNumber: u32,
    pub preloadSiblings: bool,
    pub maximumScreenSpaceError: f64,
}
impl<D: DataSource, T: TilingScheme> TileLayer<D, T> {
    pub fn new(tiling_scheme: T) -> Self {
        let ellipsoid = tiling_scheme.get_ellipsoid();
        Self {
            tiles: HashMap::new(),
            tiling_scheme: tiling_scheme,
            state: TileLayerState::Start,

            cartographicLimitRectangle: Rectangle::MAX_VALUE.clone(),
            _occluders: EllipsoidalOccluder::new(&ellipsoid),
            datasource: D::default(),
            _lastSelectionFrameNumber: 0,
            preloadSiblings: false,
            quadtree: Quadtree::default(),
            maximumScreenSpaceError: 2,
        }
    }
    pub fn get_tile_entity(&self, x: u32, y: u32, level: u32) -> Option<&Entity> {
        let key = Tile::get_key(x, y, level);
        return self.tiles.get(&key);
    }
    pub fn add_tile(&mut self, x: u32, y: u32, level: u32, entity: Entity) {
        let key = Tile::get_key(x, y, level);
        self.tiles.insert(key, entity);
    }
    pub fn is_exist(&self, x: u32, y: u32, level: u32) -> bool {
        self.get_tile_entity(x, y, level).is_some()
    }
    pub fn computeTileLoadPriority(
        &self,
        tile: &Tile,
        cameraPositionWC: &DVec3,
        cameraDirectionWC: &DVec3,
    ) -> f64 {
        let obb = tile.tileBoundingRegion.oriented_bounding_box;
        if obb.is_none() {
            return 0.0;
        }
        let mut tileDirection = obb.unwrap().center.subtract(*cameraPositionWC);
        let magnitude = tileDirection.magnitude();
        if magnitude < EPSILON5 {
            return 0.0;
        }
        tileDirection = tileDirection.divide_by_scalar(magnitude);
        return ((1.0 - tileDirection.dot(*cameraDirectionWC)) * tile._distance);
    }
    pub fn computeTileVisibility<P: Projection>(
        &self,
        tile: &mut Tile,
        cullingVolume: &CullingVolume,
        occluders: &EllipsoidalOccluder,
        cameraCartesianPosition: &DVec3,
        cameraCartographicPosition: &Cartographic,
        projection: &P,
    ) -> houtu_scene::Visibility {
        let distance = self.computeDistanceToTile(
            tile,
            occluders,
            cameraCartesianPosition,
            cameraCartographicPosition,
            projection,
        );
        tile._distance = distance;

        let undergroundVisible = false;

        let tileBoundingRegion = &tile.tileBoundingRegion;

        // if tile.boundingVolumeSourceTile == undefined {
        //     // We have no idea where self tile is, so let's just call it partially visible.
        //     return houtu_scene::Visibility::PARTIAL;
        // }

        let mut boundingVolume: Option<Box<&dyn BoundingVolume>> =
            tileBoundingRegion.oriented_bounding_box.as_ref().map(|x| {
                let bv: Box<&dyn BoundingVolume> = Box::new(x);
                return bv;
            });

        if boundingVolume.is_none() {
            boundingVolume = tileBoundingRegion.boundingSphere.as_ref().map(|x| {
                let bv: Box<&dyn BoundingVolume> = Box::new(x);
                return bv;
            });
        }

        // Check if the tile is outside the limit area in cartographic space
        tile.clippedByBoundaries = false;
        let clippedCartographicLimitRectangle =
            clipRectangleAntimeridian(&tile.rectangle, &self.cartographicLimitRectangle);
        let areaLimitIntersection =
            clippedCartographicLimitRectangle.simpleIntersection(&tile.rectangle);
        if areaLimitIntersection.is_none() {
            return houtu_scene::Visibility::NONE;
        } else {
            if !areaLimitIntersection.unwrap().eq(&tile.rectangle) {
                tile.clippedByBoundaries = true;
            }
        }
        if boundingVolume.is_none() {
            return houtu_scene::Visibility::PARTIAL;
        }

        let mut visibility = houtu_scene::Visibility::NONE;
        let intersection = cullingVolume.computeVisibility(boundingVolume.as_ref().unwrap());

        if intersection == Intersect::OUTSIDE {
            visibility = houtu_scene::Visibility::NONE;
        } else if intersection == Intersect::INTERSECTING {
            visibility = houtu_scene::Visibility::PARTIAL;
        } else if intersection == Intersect::INSIDE {
            visibility = houtu_scene::Visibility::FULL;
        }

        if visibility == houtu_scene::Visibility::NONE {
            return visibility;
        }
        if !undergroundVisible {
            let occludee_point_in_scaled_space = tile.occludee_point_in_scaled_space;
            if occludee_point_in_scaled_space.is_none() {
                return visibility;
            } else {
                if (occluders.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
                    occludee_point_in_scaled_space.as_ref().unwrap(),
                    Some(tileBoundingRegion.minimum_height),
                )) {
                    return visibility;
                }
            }

            return houtu_scene::Visibility::NONE;
        }

        return visibility;
    }
    pub fn computeDistanceToTile<P: Projection>(
        &self,
        tile: &mut Tile,
        occluders: &EllipsoidalOccluder,
        cameraCartesianPosition: &DVec3,
        cameraCartographicPosition: &Cartographic,
        projection: &P,
    ) -> f64 {
        update_tile_bounding_region(tile, occluders);

        let mut tileBoundingRegion = &mut tile.tileBoundingRegion;
        let min = tileBoundingRegion.minimum_height;
        let max = tileBoundingRegion.maximum_height;

        // if tile.boundingVolumeSourceTile != tile {
        //     let cameraHeight = cameraCartographicPosition.height;
        //     let distance_to_min = (cameraHeight - min).abs();
        //     let distance_to_max = (cameraHeight - max).abs();
        //     if distance_to_min > distance_to_max {
        //         tileBoundingRegion.minimum_height = min;
        //         tileBoundingRegion.maximum_height = min;
        //     } else {
        //         tileBoundingRegion.minimum_height = max;
        //         tileBoundingRegion.maximum_height = max;
        //     }
        // }

        let result = tileBoundingRegion.distanceToCamera(
            cameraCartesianPosition,
            cameraCartographicPosition,
            projection,
        );

        tileBoundingRegion.minimum_height = min;
        tileBoundingRegion.maximum_height = max;
        return result;
    }
}

//简化版的updateTileBoundingRegion
pub fn update_tile_bounding_region(tile: &mut Tile, occluders: &EllipsoidalOccluder) {
    let mut tileBoundingRegion = TileBoundingRegion::new(
        &tile.rectangle,
        Some(0.),
        Some(0.),
        Some(&occluders.ellipsoid),
        Some(false),
    );
    let old_minimum_height = tileBoundingRegion.minimum_height;
    let old_maximum_height = tileBoundingRegion.maximum_height;
    let mut has_bounding_volumes_from_mesh = false;

    // Get min and max heights from the mesh.
    // If the mesh is not available, get them from the terrain data.
    // If the terrain data is not available either, get them from an ancestor.
    // If none of the ancestors are available, then there are no min and max heights for self tile at self time.
    if let Some(mesh) = &tile.terrain_mesh {
        tileBoundingRegion.minimum_height = mesh.minimum_height;
        tileBoundingRegion.maximum_height = mesh.maximum_height;
        has_bounding_volumes_from_mesh = true;
        tileBoundingRegion.oriented_bounding_box = Some(mesh.oriented_bounding_box.clone());
        tileBoundingRegion.boundingSphere = Some(mesh.bounding_sphere_3d.clone());
        tile.occludee_point_in_scaled_space = mesh.occludee_point_in_scaled_space.clone();

        // If the occludee point is not defined, fallback to calculating it from the OBB
        if tile.occludee_point_in_scaled_space.is_none() {
            tile.occludee_point_in_scaled_space = compute_occludee_point(
                occluders,
                &tileBoundingRegion.oriented_bounding_box.unwrap().center,
                &tile.rectangle,
                tileBoundingRegion.minimum_height,
                tileBoundingRegion.maximum_height,
            );
        }
    }

    tile.boundingVolumeIsFromMesh = has_bounding_volumes_from_mesh;
}
pub fn compute_occludee_point(
    ellipsoidalOccluder: &EllipsoidalOccluder,
    center: &DVec3,
    rectangle: &Rectangle,
    minimum_height: f64,
    maximum_height: f64,
) -> Option<DVec3> {
    let ellipsoid = ellipsoidalOccluder.ellipsoid;

    let mut corner_positions = vec![DVec3::ZERO, DVec3::ZERO, DVec3::ZERO, DVec3::ZERO];
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

    return ellipsoidalOccluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
        center,
        &corner_positions,
        minimum_height,
    );
}
fn clipRectangleAntimeridian(
    tileRectangle: &Rectangle,
    cartographicLimitRectangle: &Rectangle,
) -> Rectangle {
    if cartographicLimitRectangle.west < cartographicLimitRectangle.east {
        return cartographicLimitRectangle.clone();
    }
    let mut splitRectangle = cartographicLimitRectangle.clone();
    let tileCenter = tileRectangle.center();
    if tileCenter.longitude > 0.0 {
        splitRectangle.east = PI;
    } else {
        splitRectangle.west = -PI;
    }
    return splitRectangle;
}
