use bevy::{
    math::DVec3,
    prelude::{AssetServer, Assets, Image},
    render::renderer::RenderDevice,
    utils::{HashMap, Uuid},
};

use houtu_jobs::JobSpawner;
use houtu_scene::{
    BoundingVolume, Cartesian3, Ellipsoid, EllipsoidalOccluder, GeographicProjection,
    GeographicTilingScheme, Intersect, Rectangle, TileBoundingRegion,
};

use crate::plugins::camera::GlobeCamera;

use super::{
    ellipsoid_terrain_provider::EllipsoidTerrainProvider,
    globe_surface_tile::{GlobeSurfaceTile, TerrainState},
    imagery::ImageryState,
    imagery_layer_storage::ImageryLayerStorage,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive::QuadtreePrimitive,
    quadtree_tile::QuadtreeTile,
    quadtree_tile_storage::QuadtreeTileStorage,
    reproject_texture::ReprojectTextureTaskQueue,
    terrain_provider::TerrainProvider,
    tile_key::TileKey,
    tile_selection_result::TileSelectionResult,
};

pub struct GlobeSurfaceTileProvider {
    terrain_provider: Box<dyn TerrainProvider>,
    readyImageryScratch: HashMap<Uuid, bool>,
    canRenderTraversalStack: Vec<TileKey>,
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
            canRenderTraversalStack: vec![],
            readyImageryScratch: HashMap::new(),
        }
    }
    pub fn load_tile(
        &mut self,
        storage: &mut QuadtreeTileStorage,
        ellipsoidal_occluder: &EllipsoidalOccluder,
        tile_key: TileKey,
        imagery_layer_storage: &mut ImageryLayerStorage,
        job_spawner: &mut JobSpawner,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        render_device: &RenderDevice,
        globe_camera: &mut GlobeCamera,
    ) {
        let tile = storage.get_mut(&tile_key).unwrap();
        let terrainStateBefore;
        let mut terrainOnly = tile.data.bounding_volume_source_tile != Some(tile_key)
            || tile.last_selection_result == TileSelectionResult::CULLED_BUT_NEEDED;
        terrainStateBefore = tile.data.terrain_state.clone();
        GlobeSurfaceTile::process_state_machine(
            storage,
            tile_key,
            &self.terrain_provider,
            imagery_layer_storage,
            terrainOnly,
            job_spawner,
            indices_and_edges_cache,
            asset_server,
            images,
            render_world_queue,
            render_device,
            globe_camera,
        );
        let tile = storage.get_mut(&tile_key).unwrap();
        if terrainOnly && terrainStateBefore != tile.data.terrain_state {
            // Terrain state changed. If:
            // a) The tile is visible, and
            // b) The bounding volume is accurate (updated as a side effect of computing visibility)
            // Then we'll load imagery, too.
            let bounding_volume_source_tile = tile.data.bounding_volume_source_tile.clone();
            if self.computeTileVisibility(storage, ellipsoidal_occluder, globe_camera, tile_key)
                != TileVisibility::NONE
                && bounding_volume_source_tile == Some(tile_key)
            {
                terrainOnly = false;
                GlobeSurfaceTile::process_state_machine(
                    storage,
                    tile_key,
                    &self.terrain_provider,
                    imagery_layer_storage,
                    terrainOnly,
                    job_spawner,
                    indices_and_edges_cache,
                    asset_server,
                    images,
                    render_world_queue,
                    render_device,
                    globe_camera,
                );
            }
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
        tile_key: TileKey,
    ) -> TileVisibility {
        computeDistanceToTile(storage, ellipsoidal_occluder, camera, tile_key);
        let tile = storage.get_mut(&tile_key).unwrap();
        let surface_tile = &mut tile.data;
        let tile_bounding_region = surface_tile
            .tile_bounding_region
            .as_ref()
            .expect("globe_surface_tile.tileBoundingRegion不存在");
        if let None = surface_tile.bounding_volume_source_tile {
            return TileVisibility::PARTIAL;
        }
        let obb = tile_bounding_region.get_bounding_volume();
        let bounding_volume: Option<Box<&dyn BoundingVolume>> = if let Some(v) = obb {
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
        let v = ellipsoidal_occluder.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
            &occludee_point_in_scaled_space,
            Some(tile_bounding_region.minimum_height),
        );

        if v {
            return visibility;
        } else {
        }

        return TileVisibility::NONE;
    }
    pub fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        return self
            .terrain_provider
            .get_level_maximum_geometric_error(level);
    }
    pub fn can_render_without_losing_detail(
        &mut self,
        tile: &mut QuadtreeTile,
        imagery_layer_storage: &mut ImageryLayerStorage,
        primitive: &mut QuadtreePrimitive,
    ) -> bool {
        let terrainReady = tile.data.terrain_state == TerrainState::READY;
        let initialImageryState = true;
        for (id, _) in imagery_layer_storage.map.iter() {
            self.readyImageryScratch
                .insert(id.clone(), initialImageryState);
        }
        for imagery in tile.data.imagery.iter_mut() {
            let is_ready = {
                let ready = imagery.loading_imagery.is_none();
                let loading_imagery = imagery.loading_imagery.as_ref().unwrap().lock();
                ready
                    || loading_imagery.state == ImageryState::FAILED
                    || loading_imagery.state == ImageryState::INVALID
            };
            let layer_id = {
                let mut id = Uuid::new_v4();
                if imagery.loading_imagery.is_some() {
                    id = imagery
                        .loading_imagery
                        .as_ref()
                        .unwrap()
                        .lock()
                        .imagery_layer_id
                }
                if imagery.ready_imagery.is_some() {
                    id = imagery
                        .ready_imagery
                        .as_ref()
                        .unwrap()
                        .lock()
                        .imagery_layer_id
                }
                id
            };
            let value = self.readyImageryScratch.get(&layer_id).unwrap();
            self.readyImageryScratch
                .insert(layer_id.clone(), is_ready && *value);
        }
        let last_frame = primitive.last_selection_frame_number;
        self.canRenderTraversalStack.clear();
        self.canRenderTraversalStack.push(tile.southwest.unwrap());
        self.canRenderTraversalStack.push(tile.southeast.unwrap());
        self.canRenderTraversalStack.push(tile.northwest.unwrap());
        self.canRenderTraversalStack.push(tile.northeast.unwrap());
        while self.canRenderTraversalStack.len() > 0 {
            let descentant_key = self.canRenderTraversalStack.pop().unwrap();
            let descendant = primitive.storage.get(&descentant_key).unwrap();
            let last_frame_selection_result =
                if descendant.last_selection_result_frame == last_frame {
                    descendant.last_selection_result.clone()
                } else {
                    TileSelectionResult::NONE
                };
            if last_frame_selection_result == TileSelectionResult::RENDERED {
                if !terrainReady && descendant.data.terrain_state == TerrainState::READY {
                    return false;
                }
                for descendant_tile_imagery in descendant.data.imagery.iter() {
                    let descendant_is_ready = {
                        let v = descendant_tile_imagery.loading_imagery.is_none();
                        let descendant_loading_imagery = descendant_tile_imagery
                            .loading_imagery
                            .as_ref()
                            .unwrap()
                            .lock();
                        v || descendant_loading_imagery.state == ImageryState::FAILED
                            || descendant_loading_imagery.state == ImageryState::INVALID
                    };
                    let descentant_layer_id = {
                        let mut id = Uuid::new_v4();
                        if descendant_tile_imagery.loading_imagery.is_some() {
                            id = descendant_tile_imagery
                                .loading_imagery
                                .as_ref()
                                .unwrap()
                                .lock()
                                .imagery_layer_id
                        }
                        if descendant_tile_imagery.ready_imagery.is_some() {
                            id = descendant_tile_imagery
                                .ready_imagery
                                .as_ref()
                                .unwrap()
                                .lock()
                                .imagery_layer_id
                        }
                        id
                    };
                    if descendant_is_ready
                        && *self.readyImageryScratch.get(&descentant_layer_id).unwrap()
                    {
                        return false;
                    }
                }
            } else if last_frame_selection_result == TileSelectionResult::REFINED {
                self.canRenderTraversalStack
                    .push(descendant.southwest.unwrap());
                self.canRenderTraversalStack
                    .push(descendant.southeast.unwrap());
                self.canRenderTraversalStack
                    .push(descendant.northwest.unwrap());
                self.canRenderTraversalStack
                    .push(descendant.northeast.unwrap());
            }
        }
        return true;
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
    tile_key: TileKey,
) -> f64 {
    update_tile_bounding_region(storage, ellipsoidal_occluder, tile_key);
    let tile = storage.get_mut(&tile_key).unwrap();
    let surface_tile = &mut tile.data;
    let bounding_volume_surface_tile = surface_tile
        .bounding_volume_source_tile
        .and_then(|x| storage.get_mut(&x));
    if let None = bounding_volume_surface_tile {
        return 9999999999.0;
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    let surface_tile = &mut tile.data;
    let tile_bounding_region = surface_tile
        .tile_bounding_region
        .as_mut()
        .expect("globe_surface_tile.tileBoundingRegion不存在");
    let min = tile_bounding_region.minimum_height;
    let max = tile_bounding_region.maximum_height;
    // let tile = storage.get_mut(&tile_key).unwrap();
    if surface_tile.bounding_volume_source_tile != Some(tile_key) {
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
    let tile = storage.get_mut(&tile_key).unwrap();
    tile.distance = distance;
    if distance < 1.0 {
        println!("too small");
    }
    return distance;
}

pub fn update_tile_bounding_region(
    storage: &mut QuadtreeTileStorage,
    ellipsoidal_occluder: &EllipsoidalOccluder,
    tile_key: TileKey,
) {
    let tile = storage.get_mut(&tile_key).unwrap();

    if let None = tile.data.tile_bounding_region {
        tile.data.tile_bounding_region = Some(TileBoundingRegion::new(
            &tile.rectangle,
            Some(0.0),
            Some(0.0),
            Some(&Ellipsoid::WGS84),
            Some(false),
        ));
    };
    let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
    let old_minimum_height = tile_bounding_region.minimum_height;
    let old_maximum_height = tile_bounding_region.maximum_height;
    let mut has_bounding_volumes_from_mesh = false;

    // let mesh = tile.data.mesh.as_ref();
    let mut ancestor_tile_key: Option<TileKey> = None;
    let mut use_ancestor_tile = false;
    if tile.data.has_mesh() {
        let cloned = tile.data.get_cloned_terrain_data();
        let terrain_data = cloned.lock().unwrap();
        let mesh = terrain_data._mesh.as_ref().unwrap();
        if let (Some(min), Some(max)) = (mesh.minimum_height, mesh.maximum_height) {
            let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
            tile_bounding_region.minimum_height = min;
            tile_bounding_region.maximum_height = max;
            has_bounding_volumes_from_mesh = true
        }
    } else {
        let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
        tile_bounding_region.minimum_height = -1.;
        tile_bounding_region.maximum_height = -1.;
        let mut ancestor_tile = tile.parent.and_then(|x| {
            ancestor_tile_key = Some(x);
            return storage.get(&x);
        });
        let mut map: HashMap<TileKey, (f64, f64)> = HashMap::new();
        while let Some(in_ancestor_tile) = ancestor_tile {
            let ancestor_surface_tile = &in_ancestor_tile.data;
            if ancestor_surface_tile.has_mesh() {
                let cloned = ancestor_surface_tile.get_cloned_terrain_data();
                let terrain_data = cloned.lock().unwrap();
                let mesh = terrain_data._mesh.as_ref().unwrap();

                map.insert(
                    tile_key,
                    (mesh.minimum_height.unwrap(), mesh.maximum_height.unwrap()),
                );
            }
            ancestor_tile = in_ancestor_tile.parent.and_then(|x| {
                ancestor_tile_key = Some(x);
                return storage.get(&x);
            });
        }
        for (key, info) in map.iter() {
            let tile = storage.get_mut(&key).unwrap();
            let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
            tile_bounding_region.minimum_height = info.0;
            tile_bounding_region.maximum_height = info.1;
        }
        use_ancestor_tile = true;
        // ancestor_tile_key = ancestor_tile.and_then(|x| Some(x.key.clone()));
    }

    if use_ancestor_tile {
        if let Some(in_source_tile_key) = ancestor_tile_key.as_ref() {
            let in_source_tile = storage.get_mut(&in_source_tile_key).unwrap();
            let in_source_tile_key = in_source_tile.key.clone();
            let tile = storage.get_mut(&tile_key).unwrap();
            //这和下面的那个has_bounding_volumes_from_mesh分支条件判断代码几乎是一样的，除了tile.data.bounding_volume_source_tile = Some(in_source_tile.key);中的in_source_tile
            if has_bounding_volumes_from_mesh {
                if !tile.data.bounding_volume_is_from_mesh {
                    let cloned = tile.data.get_cloned_terrain_data();
                    let terrain_data = cloned.lock().unwrap();
                    let mesh = terrain_data._mesh.as_ref().unwrap();
                    let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();

                    tile_bounding_region.oriented_bounding_box =
                        Some(mesh.oriented_bounding_box.clone());
                    tile_bounding_region.boundingSphere = Some(mesh.bounding_sphere_3d.clone());

                    tile.data.occludee_point_in_scaled_space =
                        mesh.occludee_point_in_scaled_space.clone();

                    if let None = tile.data.occludee_point_in_scaled_space {
                        tile.data.occludee_point_in_scaled_space = compute_occludee_point(
                            ellipsoidal_occluder,
                            &tile_bounding_region.oriented_bounding_box.unwrap().center,
                            &tile.rectangle,
                            tile_bounding_region.minimum_height,
                            tile_bounding_region.maximum_height,
                        )
                    }
                }
            } else {
                let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
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
                    tile.data.occludee_point_in_scaled_space = compute_occludee_point(
                        ellipsoidal_occluder,
                        &tile_bounding_region.oriented_bounding_box.unwrap().center,
                        &tile.rectangle,
                        tile_bounding_region.minimum_height,
                        tile_bounding_region.maximum_height,
                    )
                }
            }
            tile.data.bounding_volume_source_tile = Some(in_source_tile_key);
            tile.data.bounding_volume_is_from_mesh = has_bounding_volumes_from_mesh;
        } else {
            let tile = storage.get_mut(&tile_key).unwrap();
            tile.data.bounding_volume_source_tile = None;
            tile.data.bounding_volume_is_from_mesh = false;
        }
    } else {
        let tile = storage.get_mut(&tile_key).unwrap();
        let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
        if has_bounding_volumes_from_mesh {
            if !tile.data.bounding_volume_is_from_mesh {
                let cloned = tile.data.get_cloned_terrain_data();
                let terrain_data = cloned.lock().unwrap();
                let mesh = terrain_data._mesh.as_ref().unwrap();
                let tile_bounding_region = tile.data.tile_bounding_region.as_mut().unwrap();
                tile_bounding_region.oriented_bounding_box =
                    Some(mesh.oriented_bounding_box.clone());
                tile_bounding_region.boundingSphere = Some(mesh.bounding_sphere_3d.clone());

                tile.data.occludee_point_in_scaled_space =
                    mesh.occludee_point_in_scaled_space.clone();
                if let None = tile.data.occludee_point_in_scaled_space {
                    tile.data.occludee_point_in_scaled_space = compute_occludee_point(
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
                tile.data.occludee_point_in_scaled_space = compute_occludee_point(
                    ellipsoidal_occluder,
                    &tile_bounding_region.oriented_bounding_box.unwrap().center,
                    &tile.rectangle,
                    tile_bounding_region.minimum_height,
                    tile_bounding_region.maximum_height,
                )
            }
        }
        tile.data.bounding_volume_source_tile = Some(tile.key);
        tile.data.bounding_volume_is_from_mesh = has_bounding_volumes_from_mesh;
    }
}
fn process_source_tile(_source_tile: &mut QuadtreeTile) {}
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
