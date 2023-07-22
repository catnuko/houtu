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
    pub tileBoundingRegion: Option<TileBoundingRegion>,
    pub occludeePointInScaledSpace: Option<DVec3>,
    pub terrain_state: TerrainState,
    pub boundingVolumeIsFromMesh: bool,
    pub clippedByBoundaries: bool,
    pub mesh: Option<TerrainMesh>,
    pub terrainData: Option<Arc<Mutex<HeightmapTerrainData>>>,
    pub boundingVolumeSourceTile: Option<Entity>,
    pub vertexArray: Option<bool>, //TODO 暂时不知道放什么数据结构，先放个bool值
    pub imagery: Vec<TileImagery>,
}
impl GlobeSurfaceTile {
    pub fn new() -> Self {
        Self {
            tileBoundingRegion: None,
            occludeePointInScaledSpace: None,
            terrain_state: TerrainState::default(),
            clippedByBoundaries: false,
            boundingVolumeIsFromMesh: false,
            terrainData: None,
            mesh: None,
            boundingVolumeSourceTile: None,
            vertexArray: None,
            imagery: Vec::new(),
        }
    }
    pub fn has_mesh(&self) -> bool {
        if let Some(v) = self.terrainData.as_ref() {
            v.clone().lock().unwrap().has_mesh()
        } else {
            false
        }
    }
    pub fn get_mesh(&self) -> Option<Mesh> {
        if let Some(terrain_data) = self.terrainData.as_ref() {
            if let Some(v) = terrain_data.clone().lock().unwrap()._mesh.as_ref() {
                let mesh: Mesh = v.into();
                return Some(mesh);
            }
        }
        return None;
    }
    pub fn eligibleForUnloading(&self) -> bool {
        let loadingIsTransitioning = self.terrain_state == TerrainState::RECEIVING
            || self.terrain_state == TerrainState::TRANSFORMING;

        let shouldRemoveTile = !loadingIsTransitioning;

        //TODO
        // let imagery = self.imagery;

        // for (let i = 0, len = imagery.length; shouldRemoveTile && i < len; ++i) {
        //   let tileImagery = imagery[i];
        //   shouldRemoveTile =
        //     !defined(tileImagery.loadingImagery) ||
        //     tileImagery.loadingImagery.state != ImageryState.TRANSITIONING;
        // }

        return shouldRemoveTile;
    }
}
pub fn computeTileVisibility(
    // commands: &mut Commands,
    // ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    camera: &mut GlobeCamera,
    quadtree_tile_entity: Entity,
) -> TileVisibility {
    computeDistanceToTile(
        // commands,
        // ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
        quadtree_tile_entity,
        camera,
    );
    let (_, globe_surface_tile, _, _, _, _, _, _, _, _, _) =
        quadtree_tile_query.get(quadtree_tile_entity).unwrap();

    let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.as_ref().unwrap();

    if (globe_surface_tile.boundingVolumeSourceTile.is_none()) {
        // We have no idea where this tile is, so let's just call it partially visible.
        return TileVisibility::PARTIAL;
    }
    let obb = tileBoundingRegion.get_bounding_volume();
    let mut boundingVolume: Option<Box<&dyn BoundingVolume>> = if let Some(v) = obb {
        Some(Box::new(v))
    } else {
        if let Some(t) = tileBoundingRegion.get_bounding_sphere() {
            Some(Box::new(t))
        } else {
            None
        }
    };
    let boundingVolume = boundingVolume.unwrap();
    let mut visibility = TileVisibility::NONE;
    let intersection = camera
        .get_culling_volume()
        .computeVisibility(&boundingVolume);

    if (intersection == Intersect::OUTSIDE) {
        visibility = TileVisibility::NONE;
    } else if (intersection == Intersect::INTERSECTING) {
        visibility = TileVisibility::PARTIAL;
    } else if (intersection == Intersect::INSIDE) {
        visibility = TileVisibility::FULL;
    }

    if (visibility == TileVisibility::NONE) {
        return visibility;
    }

    let occludeePointInScaledSpace = globe_surface_tile.occludeePointInScaledSpace;
    if (occludeePointInScaledSpace.is_none()) {
        return visibility;
    }
    let occludeePointInScaledSpace = occludeePointInScaledSpace.unwrap();
    if (ellipsoidalOccluder.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
        &occludeePointInScaledSpace,
        Some(tileBoundingRegion.minimumHeight),
    )) {
        return visibility;
    }

    return TileVisibility::NONE;
}
//计算瓦片到相机的距离
fn computeDistanceToTile(
    // commands: &mut Commands,
    // ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    quadtree_tile_entity: Entity,
    camera: &mut GlobeCamera,
) {
    updateTileBoundingRegion(
        // commands,
        // ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
        quadtree_tile_entity,
    );
    let (entity, mut globe_surface_tile, _, mut other_state, _, _, _, _, _, _, _) =
        quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    let boundingVolumeSourceTile = globe_surface_tile.boundingVolumeSourceTile;
    if (boundingVolumeSourceTile.is_none()) {
        // Can't find any min/max heights anywhere? Ok, let's just say the
        // tile is really far away so we'll load and render it rather than
        // refining.
        other_state._distance = 9999999999.0;
    }

    if (globe_surface_tile.boundingVolumeSourceTile.is_some()
        && globe_surface_tile.boundingVolumeSourceTile.unwrap() != entity)
    {
        let tileBoundingRegion = globe_surface_tile
            .tileBoundingRegion
            .as_mut()
            .expect("globe_surface_tile.tileBoundingRegion不存在");
        let min = tileBoundingRegion.minimumHeight;
        let max = tileBoundingRegion.maximumHeight;

        let p = camera.get_position_cartographic();
        let distanceToMin = (p.height - min).abs();
        let distanceToMax = (p.height - max).abs();
        if (distanceToMin > distanceToMax) {
            tileBoundingRegion.minimumHeight = min;
            tileBoundingRegion.maximumHeight = min;
        } else {
            tileBoundingRegion.minimumHeight = max;
            tileBoundingRegion.maximumHeight = max;
        }
    }
    let tileBoundingRegion = globe_surface_tile
        .tileBoundingRegion
        .as_mut()
        .expect("globe_surface_tile.tileBoundingRegion不存在");
    let min = tileBoundingRegion.minimumHeight;
    let max = tileBoundingRegion.maximumHeight;
    let result = tileBoundingRegion.distanceToCamera(
        &camera.get_position_wc(),
        &camera.get_position_cartographic(),
        &GeographicProjection::WGS84,
    );

    tileBoundingRegion.minimumHeight = min;
    tileBoundingRegion.maximumHeight = max;
    other_state._distance = result;
}

pub fn updateTileBoundingRegion(
    // commands: &mut Commands,
    // ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &EllipsoidalOccluder,
    quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery>,
    quadtree_tile_entity: Entity,
) {
    let (entity, mut globe_surface_tile, rectangle, parent, _) = {
        let (entity, mut globe_surface_tile, rectangle, other_state, _, _, _, _, _, _, parent) =
            quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
        (entity, globe_surface_tile, rectangle, parent, other_state)
    };

    // let (entity, mut globe_surface_tile, rectangle, parent, other_state, _, _, _, _, _, _, _, _) =
    //     quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
    if globe_surface_tile.tileBoundingRegion.is_none() {
        globe_surface_tile.tileBoundingRegion = Some(TileBoundingRegion::new(
            rectangle,
            Some(0.0),
            Some(0.0),
            Some(&Ellipsoid::WGS84),
            Some(false),
        ));
    }

    let mut hasBoundingVolumesFromMesh = false;

    // Get min and max heights from the mesh.
    // If the mesh is not available, get them from the terrain data.
    // If the terrain data is not available either, get them from an ancestor.
    // If none of the ancestors are available, then there are no min and max heights for this tile at this time.
    let mut source_tile = Some(entity);
    if (globe_surface_tile.has_mesh()) {
        let (minimumHeight, maximumHeight) = {
            let cloned = globe_surface_tile.terrainData.as_ref().unwrap().clone();
            let terrain_data = cloned.lock().unwrap();
            let mesh = terrain_data._mesh.as_ref().unwrap();
            // let mesh = globe_surface_tile.mesh.as_ref().unwrap();
            (mesh.minimumHeight, mesh.maximumHeight)
        };
        let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.as_mut().unwrap();
        tileBoundingRegion.minimumHeight = minimumHeight;
        tileBoundingRegion.maximumHeight = maximumHeight;
        hasBoundingVolumesFromMesh = true;
    } else {
        // No accurate min/max heights available, so we're stuck with min/max heights from an ancestor tile.
        let mut ancestorTileNode = parent.clone();
        let mut minimumHeight = 0.0;
        let mut maximumHeight = 0.0;

        while let TileNode::Internal(ancestorTile) = ancestorTileNode.0 {
            let (min, max, parent_node) = {
                let (_, ancestor_globe_surface_tile, _, _, _, _, _, _, _, _, parent_node) =
                    quadtree_tile_query.get(ancestorTile).unwrap();
                // (ancestor_globe_surface_tile, parent_node)
                if (ancestor_globe_surface_tile.has_mesh()) {
                    let cloned = ancestor_globe_surface_tile
                        .terrainData
                        .as_ref()
                        .unwrap()
                        .clone();
                    let terrain_data = cloned.lock().unwrap();
                    let mesh = terrain_data._mesh.as_ref().unwrap();
                    // let t = ancestor_globe_surface_tile.mesh.as_ref().unwrap();
                    (mesh.minimumHeight, mesh.maximumHeight, parent_node)
                } else {
                    break;
                }
            };

            minimumHeight = min;
            maximumHeight = max;
            ancestorTileNode = parent_node.clone();
            source_tile = None;
        }
        let (mut globe_surface_tile,) = {
            let (_, mut globe_surface_tile, _, _, _, _, _, _, _, _, _) =
                quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile,)
        };
        let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.as_mut().unwrap();
        tileBoundingRegion.minimumHeight = minimumHeight;
        tileBoundingRegion.maximumHeight = maximumHeight;
    }

    // // Update bounding regions from the min and max heights
    if source_tile.is_some() {
        let (mut globe_surface_tile, rectangle) = {
            let (_, mut globe_surface_tile, rectangle, _, _, _, _, _, _, _, _) =
                quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile, rectangle)
        };
        if (hasBoundingVolumesFromMesh) {
            let (orientedBoundingBox, boundingSphere3D, occludeePointInScaledSpace) = {
                // let mesh = globe_surface_tile.mesh.as_ref().unwrap();
                let cloned = globe_surface_tile.terrainData.as_ref().unwrap().clone();
                let terrain_data = cloned.lock().unwrap();
                let mesh = terrain_data._mesh.as_ref().unwrap();
                (
                    mesh.orientedBoundingBox.clone(),
                    mesh.boundingSphere3D.clone(),
                    mesh.occludeePointInScaledSpace.clone(),
                )
            };
            let center = orientedBoundingBox.center.clone();
            if (!globe_surface_tile.boundingVolumeIsFromMesh) {
                let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.as_mut().unwrap();
                tileBoundingRegion.orientedBoundingBox = Some(orientedBoundingBox);
                tileBoundingRegion.boundingSphere = Some(boundingSphere3D);
                let minimumHeight = tileBoundingRegion.minimumHeight.clone();
                let maximumHeight = tileBoundingRegion.maximumHeight.clone();
                globe_surface_tile.occludeePointInScaledSpace = occludeePointInScaledSpace;

                // If the occludee point is not defined, fallback to calculating it from the OBB
                if globe_surface_tile.occludeePointInScaledSpace.is_none() {
                    globe_surface_tile.occludeePointInScaledSpace = computeOccludeePoint(
                        // &ellipsoid,
                        &ellipsoidalOccluder,
                        &center,
                        rectangle,
                        minimumHeight,
                        maximumHeight,
                    );
                }
            }
        } else {
            let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.as_mut().unwrap();
            let oldMinimumHeight = tileBoundingRegion.minimumHeight;
            let oldMaximumHeight = tileBoundingRegion.maximumHeight;
            let center = tileBoundingRegion
                .orientedBoundingBox
                .unwrap()
                .center
                .clone();

            let needsBounds = tileBoundingRegion.orientedBoundingBox.is_none()
                || tileBoundingRegion.boundingSphere.is_none();
            let heightChanged = tileBoundingRegion.minimumHeight != oldMinimumHeight
                || tileBoundingRegion.maximumHeight != oldMaximumHeight;
            if (heightChanged || needsBounds) {
                // Bounding volumes need to be recomputed in some circumstances
                tileBoundingRegion.computeBoundingVolumes(&ellipsoidalOccluder.ellipsoid);
                globe_surface_tile.occludeePointInScaledSpace = computeOccludeePoint(
                    // &ellipsoid,
                    &ellipsoidalOccluder,
                    &center,
                    rectangle,
                    tileBoundingRegion.minimumHeight,
                    tileBoundingRegion.maximumHeight,
                );
            }
        }
        let t = source_tile.clone();

        globe_surface_tile.boundingVolumeSourceTile = t;
        globe_surface_tile.boundingVolumeIsFromMesh = hasBoundingVolumesFromMesh;
    } else {
        let (mut globe_surface_tile,) = {
            let (_, mut globe_surface_tile, _, _, _, _, _, _, _, _, _) =
                quadtree_tile_query.get_mut(quadtree_tile_entity).unwrap();
            (globe_surface_tile,)
        };
        globe_surface_tile.boundingVolumeSourceTile = None;
        globe_surface_tile.boundingVolumeIsFromMesh = false;
    }
}

fn computeOccludeePoint(
    // ellipsoid: &Ellipsoid,
    ellipsoidalOccluder: &EllipsoidalOccluder,
    center: &DVec3,
    rectangle: &Rectangle,
    minimumHeight: f64,
    maximumHeight: f64,
) -> Option<DVec3> {
    let mut cornerPositions = vec![DVec3::ZERO, DVec3::ZERO, DVec3::ZERO, DVec3::ZERO];
    let ellipsoid = ellipsoidalOccluder.ellipsoid;
    cornerPositions[0] = DVec3::from_radians(
        rectangle.west,
        rectangle.south,
        Some(maximumHeight),
        Some(ellipsoid.radiiSquared),
    );
    cornerPositions[1] = DVec3::from_radians(
        rectangle.east,
        rectangle.south,
        Some(maximumHeight),
        Some(ellipsoid.radiiSquared),
    );
    cornerPositions[2] = DVec3::from_radians(
        rectangle.west,
        rectangle.north,
        Some(maximumHeight),
        Some(ellipsoid.radiiSquared),
    );
    cornerPositions[3] = DVec3::from_radians(
        rectangle.east,
        rectangle.north,
        Some(maximumHeight),
        Some(ellipsoid.radiiSquared),
    );

    return ellipsoidalOccluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
        center,
        &cornerPositions,
        minimumHeight,
    );
}
pub fn processStateMachine_system() {}
