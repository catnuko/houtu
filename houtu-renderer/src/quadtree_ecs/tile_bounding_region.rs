use crate::{
    camera::GlobeCamera,
    quadtree_ecs::{
        height_map_terrain_data::HeightmapTerrainDataCom,
        load::{Load, Queue, QueueType},
        quadtree_tile::{QuadtreeTile, TileVisibility},
    },
};
use bevy::{math::DVec3, prelude::*};
use houtu_scene::{
    BoundingVolume, Cartesian3, Ellipsoid, EllipsoidalOccluder, GeographicProjection,
    GeographicTilingScheme, Intersect, Rectangle, TileBoundingRegion, TilingScheme,
    WebMercatorTilingScheme,
};

use super::load;
#[derive(Component, Default)]
pub struct OccludeePointInScaledSpace(pub Option<DVec3>);
#[derive(Component)]
pub struct TileBoundingRegionCom {
    pub data: TileBoundingRegion,
    /// 从本节点还是父节点更新的数据？没有地形夸张时，这个值代表是否从mesh中获取boundingVolume
    pub from_mesh: bool,
    /// 如果从父节点，则是父节点的Entity
    pub parent_entity: Option<Entity>,
}
impl TileBoundingRegionCom {
    pub fn new(data: TileBoundingRegion) -> Self {
        return {
            Self {
                data: data,
                from_mesh: false,
                parent_entity: None,
            }
        };
    }
}
struct TileBoundingContext {
    entity: Entity,
    old_minimum_height: f64,
    old_maximum_height: f64,
    new_minimum_height: Option<f64>,
    new_maximum_height: Option<f64>,
    parent_entity: Option<Entity>,
    source_tile: Option<Entity>,
}
impl TileBoundingContext {
    fn new(entity: Entity, old_minimum_height: f64, old_maximum_height: f64) -> Self {
        Self {
            entity,
            old_minimum_height,
            old_maximum_height,
            new_minimum_height: None,
            new_maximum_height: None,
            parent_entity: None,
            source_tile: Some(entity),
        }
    }
}
pub fn compute_tile_visibility(
    mut globe_camera_query: Query<&mut GlobeCamera>,
    ellipsoidal_occluder: Res<EllipsoidalOccluder>,
    mut quadtree_tile_query: Query<(Entity, &mut TileBoundingRegionCom, &mut QuadtreeTile, &Load)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    for (entity, tile_bounding_region, mut quadtree_tile, load) in &mut quadtree_tile_query {
        if load.queue_type == QueueType::None {
            continue;
        }
        if tile_bounding_region.from_mesh == false && tile_bounding_region.parent_entity.is_none() {
            quadtree_tile.visibility = TileVisibility::PARTIAL;
            continue;
        }
        let obb = tile_bounding_region.data.get_bounding_volume();
        let bounding_volume: Option<Box<&dyn BoundingVolume>> = if let Some(v) = obb {
            Some(Box::new(v))
        } else {
            if let Some(t) = tile_bounding_region.data.get_bounding_sphere() {
                Some(Box::new(t))
            } else {
                None
            }
        };
        quadtree_tile.clipped_by_boundaries = false;
        if let None = bounding_volume {
            quadtree_tile.visibility = TileVisibility::PARTIAL;
            continue;
        }
        let bounding_volume = bounding_volume.unwrap();
        let mut visibility = TileVisibility::NONE;
        let intersection = globe_camera
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
            quadtree_tile.visibility = visibility;
            continue;
        }

        let occludee_point_in_scaled_space = quadtree_tile.occludee_point_in_scaled_space;
        if occludee_point_in_scaled_space.is_none() {
            quadtree_tile.visibility = visibility;
            continue;
        }
        let occludee_point_in_scaled_space = occludee_point_in_scaled_space.unwrap();
        let v = ellipsoidal_occluder.is_scaled_space_point_visible_possibly_under_ellipsoid(
            &occludee_point_in_scaled_space,
            Some(tile_bounding_region.data.minimum_height),
        );
        if v {
            quadtree_tile.visibility = visibility;
            continue;
        }
        quadtree_tile.visibility = TileVisibility::NONE;
    }
}
pub fn compute_distance_to_tile(
    mut globe_camera_query: Query<&mut GlobeCamera>,
    mut quadtree_tile_query: Query<(Entity, &mut TileBoundingRegionCom, &mut QuadtreeTile, &Load)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");

    for (entity, mut tile_bounding_region, mut quadtree_tile, load) in &mut quadtree_tile_query {
        if load.queue_type == QueueType::None {
            continue;
        }
        if tile_bounding_region.from_mesh == false && tile_bounding_region.parent_entity.is_none() {
            quadtree_tile.distance = f64::MAX;
            continue;
        }
        let min = tile_bounding_region.data.minimum_height;
        let max = tile_bounding_region.data.maximum_height;
        if tile_bounding_region.parent_entity != Some(entity) {
            let p = globe_camera.get_position_cartographic();
            let distance_to_min = (p.height - min).abs();
            let distance_to_max = (p.height - max).abs();
            if distance_to_min > distance_to_max {
                tile_bounding_region.data.minimum_height = min;
                tile_bounding_region.data.maximum_height = min;
            } else {
                tile_bounding_region.data.minimum_height = max;
                tile_bounding_region.data.maximum_height = max;
            }
        }
        tile_bounding_region.data.minimum_height = min;
        tile_bounding_region.data.maximum_height = min;

        quadtree_tile.distance = tile_bounding_region.data.distance_to_camera_region(
            &globe_camera.get_position_wc(),
            &globe_camera.get_position_cartographic(),
            &GeographicProjection::WGS84,
        );
    }
}
pub fn update_tile_bounding_region(
    ellipsoid: Res<Ellipsoid>,
    ellipsoidal_occluder: Res<EllipsoidalOccluder>,
    mut quadtree_tile_query: Query<(
        Entity,
        &mut TileBoundingRegionCom,
        Option<&HeightmapTerrainDataCom>,
        &Load,
        &Rectangle,
        &mut QuadtreeTile,
    )>,
    parent_query: Query<&Parent>,
) {
    let mut entities = vec![];
    for (
        entity,
        mut tile_bounding_region,
        height_map_terrain_data_option,
        load,
        rectangle,
        mut quadtree_tile,
    ) in &mut quadtree_tile_query
    {
        if load.queue_type == QueueType::None || tile_bounding_region.from_mesh {
            continue;
        }
        if let Some(height_map_terrain_data) = height_map_terrain_data_option {
            if height_map_terrain_data.0._mesh.is_some() {
                let mesh = height_map_terrain_data.0._mesh.as_ref().unwrap();
                tile_bounding_region.data.minimum_height = mesh.minimum_height;
                tile_bounding_region.data.maximum_height = mesh.maximum_height;
                tile_bounding_region.from_mesh = true;

                tile_bounding_region.data.oriented_bounding_box =
                    Some(mesh.oriented_bounding_box.clone());
                tile_bounding_region.data.bounding_sphere = Some(mesh.bounding_sphere_3d.clone());
                quadtree_tile.occludee_point_in_scaled_space =
                    if let Some(p) = mesh.occludee_point_in_scaled_space {
                        Some(p.clone())
                    } else {
                        compute_occludee_point(
                            &ellipsoidal_occluder,
                            &tile_bounding_region
                                .data
                                .oriented_bounding_box
                                .unwrap()
                                .center,
                            rectangle,
                            tile_bounding_region.data.minimum_height,
                            tile_bounding_region.data.maximum_height,
                        )
                    };
            }
        }
        // 需要从最近的父节点中更新数据
        if !tile_bounding_region.from_mesh {
            let old_minimum_height = tile_bounding_region.data.minimum_height;
            let old_maximum_height = tile_bounding_region.data.maximum_height;
            tile_bounding_region.data.minimum_height = -1.;
            tile_bounding_region.data.maximum_height = -1.;
            entities.push(TileBoundingContext::new(
                entity,
                old_minimum_height,
                old_maximum_height,
            ));
        }
    }
    for ctx in &mut entities {
        let mut parent_iter = parent_query.iter_ancestors(ctx.entity);
        // 如果是根节点，parent不存在，则parent_entity为空
        while let Some(parent_entity) = parent_iter.next() {
            let (_, _, height_map_terrain_data_option, _, _, _) =
                quadtree_tile_query.get_mut(parent_entity).unwrap();
            if let Some(height_map_terrain_data) = height_map_terrain_data_option {
                if height_map_terrain_data.0._mesh.is_some() {
                    let mesh = height_map_terrain_data.0._mesh.as_ref().unwrap();
                    ctx.new_minimum_height = Some(mesh.minimum_height);
                    ctx.new_maximum_height = Some(mesh.maximum_height);
                    ctx.parent_entity = Some(parent_entity);
                    break;
                }
            }
        }
        let (_, mut tile_bounding_region, _, _, rectangle, mut quadtree_tile) =
            quadtree_tile_query.get_mut(ctx.entity).unwrap();
        if let (Some(min), Some(max)) = (ctx.new_minimum_height, ctx.new_maximum_height) {
            tile_bounding_region.data.minimum_height = min;
            tile_bounding_region.data.maximum_height = max;
            tile_bounding_region.from_mesh = true;
            tile_bounding_region.parent_entity = ctx.parent_entity.clone();
            let needs_bounds = tile_bounding_region.data.oriented_bounding_box.is_some()
                && tile_bounding_region.data.bounding_sphere.is_some();
            let height_changed = tile_bounding_region.data.minimum_height != ctx.old_minimum_height
                || tile_bounding_region.data.maximum_height != ctx.old_maximum_height;
            if height_changed || needs_bounds {
                tile_bounding_region
                    .data
                    .compute_bounding_volumes(&ellipsoid);
                quadtree_tile.occludee_point_in_scaled_space = compute_occludee_point(
                    &ellipsoidal_occluder,
                    &tile_bounding_region
                        .data
                        .oriented_bounding_box
                        .unwrap()
                        .center,
                    rectangle,
                    tile_bounding_region.data.minimum_height,
                    tile_bounding_region.data.maximum_height,
                )
            }
        } else {
            tile_bounding_region.parent_entity = None;
            tile_bounding_region.from_mesh = false;
        }
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

    return ellipsoidal_occluder.compute_horizon_culling_point_possibly_under_ellipsoid(
        center,
        &corner_positions,
        minimum_height,
    );
}
