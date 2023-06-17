use bevy::{math::DVec3, prelude::*};
use houtu_scene::{
    BoundingVolume, Cartesian3, CullingVolume, Ellipsoid, EllipsoidalOccluder,
    GeographicProjection, HeightmapTerrainData, Intersect, Rectangle, TerrainMesh,
    TileBoundingRegion,
};

use crate::plugins::camera::GlobeCamera;

use super::quadtree_tile::{QuadtreeTileMark, QuadtreeTileOtherState, TileNode};
#[derive(Debug)]
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
#[derive(Component, Debug)]
pub struct GlobeSurfaceTile {
    pub tileBoundingRegion: Option<TileBoundingRegion>,
    pub occludeePointInScaledSpace: Option<DVec3>,
    pub terrain_state: TerrainState,
    pub boundingVolumeIsFromMesh: bool,
    pub clippedByBoundaries: bool,
    pub mesh: Option<TerrainMesh>,
    pub terrainData: Option<HeightmapTerrainData>,
    pub boundingVolumeSourceTile: Option<Entity>,
    pub vertexArray: Option<bool>, //TODO 暂时不知道放什么数据结构，先放个bool值
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
        }
    }
}
pub fn computeTileVisibility(
    commands: &mut Commands,
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: (
        Entity,
        &GlobeSurfaceTile,
        &Rectangle,
        &TileNode,
        &QuadtreeTileOtherState,
    ),
    camera: &mut GlobeCamera,
    culling_volume: &CullingVolume,
) -> TileVisibility {
    computeDistanceToTile(
        commands,
        ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
        camera,
    );
    let (entity, globe_surface_tile, rectangle, parent, mut other_state) = quadtree_tile_query;

    let tileBoundingRegion = globe_surface_tile.tileBoundingRegion.unwrap();

    if (globe_surface_tile.boundingVolumeSourceTile.is_none()) {
        // We have no idea where this tile is, so let's just call it partially visible.
        return TileVisibility::PARTIAL;
    }
    let mut boundingVolume: Option<Box<&dyn BoundingVolume>> = None;
    let obb = tileBoundingRegion.get_bounding_volume();
    if let Some(v) = obb {
        boundingVolume = Some(Box::new(&v));
    } else {
        let sp = tileBoundingRegion.get_bounding_sphere();
        if let Some(t) = sp {
            boundingVolume = Some(Box::new(&t));
        }
    }
    let boundingVolume = boundingVolume.unwrap();
    let visibility;
    let intersection = culling_volume.computeVisibility(&boundingVolume);

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
    commands: &mut Commands,
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: (
        Entity,
        &GlobeSurfaceTile,
        &Rectangle,
        &TileNode,
        &QuadtreeTileOtherState,
    ),
    camera: &mut GlobeCamera,
) {
    updateTileBoundingRegion(
        commands,
        ellipsoid,
        ellipsoidalOccluder,
        quadtree_tile_query,
    );
    let (entity, globe_surface_tile, rectangle, parent, mut other_state) = quadtree_tile_query;
    let boundingVolumeSourceTile = globe_surface_tile.boundingVolumeSourceTile;
    if (boundingVolumeSourceTile.is_none()) {
        // Can't find any min/max heights anywhere? Ok, let's just say the
        // tile is really far away so we'll load and render it rather than
        // refining.
        other_state._distance = 9999999999.0;
    }

    let mut tileBoundingRegion = globe_surface_tile
        .tileBoundingRegion
        .expect("globe_surface_tile.tileBoundingRegion不存在");
    let min = tileBoundingRegion.minimumHeight;
    let max = tileBoundingRegion.maximumHeight;

    if (globe_surface_tile.boundingVolumeSourceTile.is_some()
        && globe_surface_tile.boundingVolumeSourceTile.unwrap() != entity)
    {
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
    commands: &mut Commands,
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    quadtree_tile_query: (
        Entity,
        &GlobeSurfaceTile,
        &Rectangle,
        &TileNode,
        &QuadtreeTileOtherState,
    ),
) {
    let (entity, globe_surface_tile, rectangle, parent, other_state) = quadtree_tile_query;
    if globe_surface_tile.tileBoundingRegion.is_none() {
        globe_surface_tile.tileBoundingRegion = Some(TileBoundingRegion::new(
            rectangle,
            Some(0.0),
            Some(0.0),
            Some(&Ellipsoid::WGS84),
            Some(false),
        ));
    }
    let mut tileBoundingRegion = globe_surface_tile.tileBoundingRegion.unwrap();
    let oldMinimumHeight = tileBoundingRegion.minimumHeight;
    let oldMaximumHeight = tileBoundingRegion.maximumHeight;
    let mut hasBoundingVolumesFromMesh = false;

    // Get min and max heights from the mesh.
    // If the mesh is not available, get them from the terrain data.
    // If the terrain data is not available either, get them from an ancestor.
    // If none of the ancestors are available, then there are no min and max heights for this tile at this time.
    let mesh = globe_surface_tile.mesh;
    let terrainData = globe_surface_tile.terrainData;
    let mut source_tile = Some(entity);
    if (mesh.is_some()) {
        let mesh = mesh.unwrap();
        tileBoundingRegion.minimumHeight = mesh.minimumHeight;
        tileBoundingRegion.maximumHeight = mesh.maximumHeight;
        hasBoundingVolumesFromMesh = true;
    } else {
        // No accurate min/max heights available, so we're stuck with min/max heights from an ancestor tile.
        let mut ancestorTileNode = parent.clone();
        while let TileNode::Internal(ancestorTile) = ancestorTileNode {
            let mut is_pass = false;
            commands.add(|world: &mut World| {
                let ancestor_globe_surface_tile = world.get::<GlobeSurfaceTile>(ancestorTile);
                let parent_node = world.get::<TileNode>(ancestorTile);
                if let Some(v) = ancestor_globe_surface_tile {
                    let ancestorMesh = v.mesh;
                    if (ancestorMesh.is_some()) {
                        let t = ancestorMesh.unwrap();
                        tileBoundingRegion.minimumHeight = t.minimumHeight;
                        tileBoundingRegion.maximumHeight = t.maximumHeight;
                        is_pass = true;
                        ancestorTileNode = parent_node.unwrap().clone();
                        source_tile = None;
                    }
                }
            });
            if is_pass {
                break;
            }
        }
    }

    // Update bounding regions from the min and max heights
    if source_tile.is_some() {
        if (hasBoundingVolumesFromMesh) {
            let mesh = mesh.unwrap();
            if (!globe_surface_tile.boundingVolumeIsFromMesh) {
                tileBoundingRegion.orientedBoundingBox = Some(mesh.orientedBoundingBox.clone());
                tileBoundingRegion.boundingSphere = Some(mesh.boundingSphere3D.clone());
                globe_surface_tile.occludeePointInScaledSpace =
                    mesh.occludeePointInScaledSpace.clone();

                // If the occludee point is not defined, fallback to calculating it from the OBB
                if (globe_surface_tile.occludeePointInScaledSpace.is_none()) {
                    globe_surface_tile.occludeePointInScaledSpace = computeOccludeePoint(
                        &ellipsoid,
                        &ellipsoidalOccluder,
                        &tileBoundingRegion
                            .orientedBoundingBox
                            .expect("希望orientedBoundingBox存在")
                            .center,
                        rectangle,
                        tileBoundingRegion.minimumHeight,
                        tileBoundingRegion.maximumHeight,
                    );
                }
            }
        } else {
            let needsBounds = tileBoundingRegion.orientedBoundingBox.is_none()
                || tileBoundingRegion.boundingSphere.is_none();
            let heightChanged = tileBoundingRegion.minimumHeight != oldMinimumHeight
                || tileBoundingRegion.maximumHeight != oldMaximumHeight;
            if (heightChanged || needsBounds) {
                // Bounding volumes need to be recomputed in some circumstances
                tileBoundingRegion.computeBoundingVolumes(&ellipsoid);
                globe_surface_tile.occludeePointInScaledSpace = computeOccludeePoint(
                    &ellipsoid,
                    &ellipsoidalOccluder,
                    &tileBoundingRegion
                        .orientedBoundingBox
                        .expect("希望orientedBoundingBox存在")
                        .center,
                    rectangle,
                    tileBoundingRegion.minimumHeight,
                    tileBoundingRegion.maximumHeight,
                );
            }
        }
        globe_surface_tile.boundingVolumeSourceTile = source_tile;
        globe_surface_tile.boundingVolumeIsFromMesh = hasBoundingVolumesFromMesh;
    } else {
        globe_surface_tile.boundingVolumeSourceTile = None;
        globe_surface_tile.boundingVolumeIsFromMesh = false;
    }
}

fn computeOccludeePoint(
    ellipsoid: &Res<Ellipsoid>,
    ellipsoidalOccluder: &Res<EllipsoidalOccluder>,
    center: &DVec3,
    rectangle: &Rectangle,
    minimumHeight: f64,
    maximumHeight: f64,
) -> Option<DVec3> {
    let mut cornerPositions = vec![DVec3::ZERO, DVec3::ZERO, DVec3::ZERO, DVec3::ZERO];
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
