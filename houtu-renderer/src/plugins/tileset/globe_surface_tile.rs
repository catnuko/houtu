use std::sync::{Arc, Mutex, MutexGuard};

use bevy::{math::DVec3, prelude::*};
use houtu_scene::{
    BoundingVolume, Cartesian3, CullingVolume, Ellipsoid, EllipsoidalOccluder,
    GeographicProjection, HeightmapTerrainData, Intersect, Rectangle, TerrainMesh,
    TileBoundingRegion,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    imagery::TileImagery,
    quadtree_tile::{QuadtreeTileMark, QuadtreeTileOtherState, TileNode},
    tile_quad_tree::GlobeSurfaceTileQuery,
};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TerrainState {
    FAILED = 0,
    UNLOADED = 1,
    RECEIVING = 2,
    RECEIVED = 3,
    TRANSFORMING = 4,
    TRANSFORMED = 5,
    READY = 6,
}
impl Default for TerrainState {
    fn default() -> Self {
        Self::UNLOADED
    }
}
#[derive(Debug, PartialEq)]
pub enum TileVisibility {
    NONE = -1,
    PARTIAL = 0,
    FULL = 1,
}
#[derive(Component)]
pub struct GlobeSurfaceTile {
    pub tile_bounding_region: Option<TileBoundingRegion>,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub terrain_state: TerrainState,
    pub bounding_volume_is_from_mesh: bool,
    pub clipped_by_boundaries: bool,
    pub mesh: Option<TerrainMesh>,
    pub bounding_volume_source_tile: Option<Entity>,
    pub vertex_array: Option<bool>, //TODO 暂时不知道放什么数据结构，先放个bool值
    pub imagery: Vec<TileImagery>,
}
impl GlobeSurfaceTile {
    pub fn new() -> Self {
        Self {
            tile_bounding_region: None,
            occludee_point_in_scaled_space: None,
            terrain_state: TerrainState::default(),
            clipped_by_boundaries: false,
            bounding_volume_is_from_mesh: false,
            mesh: None,
            bounding_volume_source_tile: None,
            vertex_array: None,
            imagery: Vec::new(),
        }
    }
    pub fn eligible_for_unloading(&self) -> bool {
        let loading_is_transitioning = self.terrain_state == TerrainState::RECEIVING
            || self.terrain_state == TerrainState::TRANSFORMING;

        let should_removeTile = !loading_is_transitioning;

        //TODO
        // let imagery = self.imagery;

        // for (let i = 0, len = imagery.length; should_removeTile && i < len; ++i) {
        //   let tile_imagery = imagery[i];
        //   should_removeTile =
        //     !defined(tile_imagery.loading_imagery) ||
        //     tile_imagery.loading_imagery.state != ImageryState.TRANSITIONING;
        // }

        return should_removeTile;
    }
}
pub fn computeTileVisibility(
    ellipsoidal_occluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    camera: &mut GlobeCamera,
    quadtree_tile_entity: Entity,
) -> TileVisibility {
    computeDistanceToTile(
        ellipsoidal_occluder,
        quadtree_tile_query,
        quadtree_tile_entity,
        camera,
    );
    let (_, globe_surface_tile, _, _, _, _, _, _, _, _, _, terrain_datasource_data) =
        quadtree_tile_query.get(quadtree_tile_entity).unwrap();

    let tile_bounding_region = globe_surface_tile.tile_bounding_region.as_ref().unwrap();

    if globe_surface_tile.bounding_volume_source_tile.is_none() {
        // We have no idea where this tile is, so let's just call it partially visible.
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

    let occludee_point_in_scaled_space = globe_surface_tile.occludee_point_in_scaled_space;
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

//计算瓦片到相机的距离
fn computeDistanceToTile(
    ellipsoidal_occluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    quadtree_tile_entity: Entity,
    camera: &mut GlobeCamera,
) {
    update_tile_bounding_region(
        ellipsoidal_occluder,
        quadtree_tile_query,
        quadtree_tile_entity,
    );
    let (
        entity,
        mut globe_surface_tile,
        _,
        mut other_state,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        terrain_datasource_data,
    ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let bounding_volume_source_tile = globe_surface_tile.bounding_volume_source_tile;
    if bounding_volume_source_tile.is_none() {
        // Can't find any min/max heights anywhere? Ok, let's just say the
        // tile is really far away so we'll load and render it rather than
        // refining.
        other_state._distance = 9999999999.0;
    }

    if globe_surface_tile.bounding_volume_source_tile.is_some()
        && globe_surface_tile.bounding_volume_source_tile.unwrap() != entity
    {
        let tile_bounding_region = globe_surface_tile
            .tile_bounding_region
            .as_mut()
            .expect("globe_surface_tile.tileBoundingRegion不存在");
        let min = tile_bounding_region.minimum_height;
        let max = tile_bounding_region.maximum_height;

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
    let tile_bounding_region = globe_surface_tile
        .tile_bounding_region
        .as_mut()
        .expect("globe_surface_tile.tileBoundingRegion不存在");
    let min = tile_bounding_region.minimum_height;
    let max = tile_bounding_region.maximum_height;
    let result = tile_bounding_region.distanceToCamera(
        &camera.get_position_wc(),
        &camera.get_position_cartographic(),
        &GeographicProjection::WGS84,
    );

    tile_bounding_region.minimum_height = min;
    tile_bounding_region.maximum_height = max;
    other_state._distance = result;
}

pub fn update_tile_bounding_region(
    ellipsoidal_occluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    quadtree_tile_entity: Entity,
) {
    let (entity, mut globe_surface_tile, rectangle, parent, _, terrain_datasource_data) = {
        let (
            entity,
            mut globe_surface_tile,
            rectangle,
            other_state,
            _,
            _,
            _,
            _,
            _,
            _,
            parent,
            terrain_datasource_data,
        ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
        (
            entity,
            globe_surface_tile,
            rectangle,
            parent,
            other_state,
            terrain_datasource_data,
        )
    };

    // let (entity, mut globe_surface_tile, rectangle, parent, other_state, _, _, _, _, _, _, _, _) =
    //     quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    if globe_surface_tile.tile_bounding_region.is_none() {
        globe_surface_tile.tile_bounding_region = Some(TileBoundingRegion::new(
            rectangle,
            Some(0.0),
            Some(0.0),
            Some(&Ellipsoid::WGS84),
            Some(false),
        ));
    }

    let mut has_bounding_volumes_from_mesh = false;

    // Get min and max heights from the mesh.
    // If the mesh is not available, get them from the terrain data.
    // If the terrain data is not available either, get them from an ancestor.
    // If none of the ancestors are available, then there are no min and max heights for this tile at this time.
    let mut source_tile = Some(entity);
    if terrain_datasource_data.has_mesh() {
        let (minimum_height, maximum_height) = {
            let cloned = terrain_datasource_data.get_cloned_terrain_data();
            let terrain_data = cloned.lock().unwrap();
            let mesh = terrain_data._mesh.as_ref().unwrap();
            // let mesh = globe_surface_tile.mesh.as_ref().unwrap();
            (mesh.minimum_height, mesh.maximum_height)
        };
        let tile_bounding_region = globe_surface_tile.tile_bounding_region.as_mut().unwrap();
        tile_bounding_region.minimum_height = minimum_height.unwrap();
        tile_bounding_region.maximum_height = maximum_height.unwrap();
        has_bounding_volumes_from_mesh = true;
    } else {
        // No accurate min/max heights available, so we're stuck with min/max heights from an ancestor tile.
        let mut ancestor_tile_node = parent.clone();
        let mut minimum_height = 0.0;
        let mut maximum_height = 0.0;

        while let TileNode::Internal(ancestor_tile) = ancestor_tile_node.0 {
            let (min, max, parent_node) = {
                let (
                    _,
                    ancestor_globe_surface_tile,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    parent_node,
                    terrain_datasource_data,
                ) = quadtree_tile_query.get(ancestor_tile).unwrap();
                // (ancestor_globe_surface_tile, parent_node)
                if terrain_datasource_data.has_mesh() {
                    let cloned = terrain_datasource_data.get_cloned_terrain_data();
                    let terrain_data = cloned.lock().unwrap();
                    let mesh = terrain_data._mesh.as_ref().unwrap();
                    // let t = ancestor_globe_surface_tile.mesh.as_ref().unwrap();
                    (mesh.minimum_height, mesh.maximum_height, parent_node)
                } else {
                    break;
                }
            };

            minimum_height = min.unwrap();
            maximum_height = max.unwrap();
            ancestor_tile_node = parent_node.clone();
            source_tile = None;
        }
        let (mut globe_surface_tile,) = {
            let (_, mut globe_surface_tile, _, _, _, _, _, _, _, _, _, terrain_datasource_data) =
                quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile,)
        };
        let tile_bounding_region = globe_surface_tile.tile_bounding_region.as_mut().unwrap();
        tile_bounding_region.minimum_height = minimum_height;
        tile_bounding_region.maximum_height = maximum_height;
    }

    // // Update bounding regions from the min and max heights
    if source_tile.is_some() {
        let (mut globe_surface_tile, rectangle, terrain_datasource_data) = {
            let (
                _,
                mut globe_surface_tile,
                rectangle,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
                terrain_datasource_data,
            ) = quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile, rectangle, terrain_datasource_data)
        };
        if has_bounding_volumes_from_mesh {
            let (oriented_bounding_box, bounding_sphere_3d, occludee_point_in_scaled_space) = {
                // let mesh = globe_surface_tile.mesh.as_ref().unwrap();
                let cloned = terrain_datasource_data.get_cloned_terrain_data();
                let terrain_data = cloned.lock().unwrap();
                let mesh = terrain_data._mesh.as_ref().unwrap();
                (
                    mesh.oriented_bounding_box.clone(),
                    mesh.bounding_sphere_3d.clone(),
                    mesh.occludee_point_in_scaled_space.clone(),
                )
            };
            let center = oriented_bounding_box.center.clone();
            if !globe_surface_tile.bounding_volume_is_from_mesh {
                let tile_bounding_region =
                    globe_surface_tile.tile_bounding_region.as_mut().unwrap();
                tile_bounding_region.oriented_bounding_box = Some(oriented_bounding_box);
                tile_bounding_region.boundingSphere = Some(bounding_sphere_3d);
                let minimum_height = tile_bounding_region.minimum_height.clone();
                let maximum_height = tile_bounding_region.maximum_height.clone();
                globe_surface_tile.occludee_point_in_scaled_space = occludee_point_in_scaled_space;

                // If the occludee point is not defined, fallback to calculating it from the OBB
                if globe_surface_tile.occludee_point_in_scaled_space.is_none() {
                    globe_surface_tile.occludee_point_in_scaled_space = compute_occludee_point(
                        // &ellipsoid,
                        &ellipsoidal_occluder,
                        &center,
                        rectangle,
                        minimum_height,
                        maximum_height,
                    );
                }
            }
        } else {
            let tile_bounding_region = globe_surface_tile.tile_bounding_region.as_mut().unwrap();
            let old_minimum_height = tile_bounding_region.minimum_height;
            let old_maximum_height = tile_bounding_region.maximum_height;
            let center = tile_bounding_region
                .oriented_bounding_box
                .unwrap()
                .center
                .clone();

            let needs_bounds = tile_bounding_region.oriented_bounding_box.is_none()
                || tile_bounding_region.boundingSphere.is_none();
            let height_changed = tile_bounding_region.minimum_height != old_minimum_height
                || tile_bounding_region.maximum_height != old_maximum_height;
            if height_changed || needs_bounds {
                // Bounding volumes need to be recomputed in some circumstances
                tile_bounding_region.computeBoundingVolumes(&ellipsoidal_occluder.ellipsoid);
                globe_surface_tile.occludee_point_in_scaled_space = compute_occludee_point(
                    // &ellipsoid,
                    &ellipsoidal_occluder,
                    &center,
                    rectangle,
                    tile_bounding_region.minimum_height,
                    tile_bounding_region.maximum_height,
                );
            }
        }
        let t = source_tile.clone();

        globe_surface_tile.bounding_volume_source_tile = t;
        globe_surface_tile.bounding_volume_is_from_mesh = has_bounding_volumes_from_mesh;
    } else {
        let (mut globe_surface_tile,) = {
            let (_, mut globe_surface_tile, _, _, _, _, _, _, _, _, _, terrain_datasource_data) =
                quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile,)
        };
        globe_surface_tile.bounding_volume_source_tile = None;
        globe_surface_tile.bounding_volume_is_from_mesh = false;
    }
}

fn compute_occludee_point(
    // ellipsoid: &Ellipsoid,
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
pub fn processStateMachine_system() {}
