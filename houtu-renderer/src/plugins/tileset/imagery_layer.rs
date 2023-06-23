use std::{
    f64::consts::{E, PI},
    sync::Arc,
};

use bevy::{math::DVec4, prelude::*, utils::HashMap};
use houtu_scene::{
    Ellipsoid, GeographicTilingScheme, HeightmapTerrainData, Rectangle, TilingScheme,
};

use super::{
    imagery::{Imagery, TileImagery},
    quadtree_tile::TileToLoad,
    tile_quad_tree::GlobeSurfaceTileQuery,
    TileKey,
};
#[derive(Debug, Component)]
pub struct ImageryLayerOtherState {
    pub minimumTerrainLevel: Option<u32>,
    pub maximumTerrainLevel: Option<u32>,
}
#[derive(Component)]
pub struct ImageryLayer {
    pub alpha: f64,
    pub nightAlpha: f64,
    pub dayAlpha: f64,
    pub brightness: f64,
    pub contrast: f64,
    pub hue: f64,
    pub saturation: f64,
    pub gamma: f64,
    pub z_index: u32,
    pub _isBaseLayer: bool,
    pub ready: bool,
    pub cutoutRectangle: Option<Rectangle>,
    pub colorToAlpha: f64,
    pub colorToAlphaThreshold: f64,
    pub _rectangle: Rectangle,
    pub _skeletonPlaceholder: TileImagery,
    pub _imageryCache: HashMap<String, Arc<Imagery>>,
}
impl ImageryLayer {
    pub fn new(imagery_layer_entity: Entity) -> Self {
        Self {
            alpha: 1.0,
            nightAlpha: 1.0,
            dayAlpha: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            hue: 0.0,
            saturation: 1.0,
            gamma: 1.0,
            z_index: 0,
            _isBaseLayer: false,
            cutoutRectangle: None,
            colorToAlpha: 1.0,
            colorToAlphaThreshold: 0.004,
            ready: true,
            _rectangle: Rectangle::MAX_VALUE.clone(),
            _skeletonPlaceholder: TileImagery::createPlaceholder(imagery_layer_entity, None),
            _imageryCache: HashMap::new(),
        }
    }
}
impl ImageryLayer {
    fn getLevelWithMaximumTexelSpacing(
        &mut self,
        texelSpacing: f64,
        latitudeClosestToEquator: f64,
        imageryProvider: &mut XYZDataSource,
    ) -> u32 {
        // PERFORMANCE_IDEA: factor out the stuff that doesn't change.
        let tilingScheme = imageryProvider.tiling_scheme;
        let ellipsoid = tilingScheme.ellipsoid;
        let latitudeFactor = if false {
            latitudeClosestToEquator.cos()
        } else {
            1.0
        };
        let tilingSchemeRectangle = tilingScheme.rectangle;
        let levelZeroMaximumTexelSpacing = (ellipsoid.maximumRadius
            * tilingSchemeRectangle.computeWidth()
            * latitudeFactor)
            / (imageryProvider.tile_width * tilingScheme.get_number_of_x_tiles_at_level(0)) as f64;

        let twoToTheLevelPower = levelZeroMaximumTexelSpacing / texelSpacing;
        let level = E.log(twoToTheLevelPower) / E.log(2.);
        let rounded = level.round() as u32;
        return rounded | 0;
    }

    fn isBaseLayer(&self) -> bool {
        return self._isBaseLayer;
    }
    pub fn _createTileImagerySkeletons(
        &self,
        quadtree_tile_query: &mut Query<GlobeSurfaceTileQuery, With<TileToLoad>>,
        globe_surface_tile_entity: Entity,
        terrain_datasource: &mut TerrainDataSource,
        imagery_layer_query: &mut Query<(
            Entity,
            &mut Visibility,
            &mut ImageryLayer,
            &mut XYZDataSource,
        )>,
        imagery_layer_entity: Entity,
    ) -> bool {
        let (
            entity,
            mut globe_surface_tile,
            rectangle,
            mut other_state,
            mut replacement_state,
            key,
            node_id,
            mut node_children,
            mut state,
            location,
            parent,
        ) = quadtree_tile_query
            .get_mut(globe_surface_tile_entity)
            .unwrap();
        let (_, visibility, _, mut imagery_datasource) =
            imagery_layer_query.get_mut(imagery_layer_entity).unwrap();
        let mut insertionPoint = globe_surface_tile.imagery.len();

        // ready is deprecated. This is here for backwards compatibility
        if (!self.ready || !imagery_datasource.ready) {
            // The imagery provider is not ready, so we can't create skeletons, yet.
            // Instead, add a placeholder so that we'll know to create
            // the skeletons once the provider is ready.
            // self._skeletonPlaceholder.loadingImagery.addReference();
            globe_surface_tile
                .imagery
                .insert(insertionPoint, self._skeletonPlaceholder);
            return true;
        }

        // Use Web Mercator for our texture coordinate computations if this imagery layer uses
        // that projection and the terrain tile falls entirely inside the valid bounds of the
        // projection.
        let useWebMercatorT = false;

        // Compute the rectangle of the imagery from this imagery_datasource that overlaps
        // the geometry tile.  The imagery_datasource and ImageryLayer both have the
        // opportunity to letrain the rectangle.  The imagery TilingScheme's rectangle
        // always fully contains the imagery_datasource's rectangle.
        let imageryBounds = imagery_datasource
            .rectangle
            .intersection(&self._rectangle)
            .expect("多边形相交没结果");
        let mut intersection_rectangle = rectangle.intersection(&imageryBounds);

        if (intersection_rectangle.is_none()) {
            // There is no overlap between this terrain tile and this imagery
            // provider.  Unless this is the base layer, no skeletons need to be created.
            // We stretch texels at the edge of the base layer over the entire globe.
            if (!self._isBaseLayer) {
                return false;
            }

            let baseImageryRectangle = imageryBounds;
            let baseTerrainRectangle = rectangle;
            let mut new_rectangle = Rectangle::default();

            if (baseTerrainRectangle.south >= baseImageryRectangle.north) {
                new_rectangle.north = baseImageryRectangle.north;
                new_rectangle.south = baseImageryRectangle.north;
            } else if (baseTerrainRectangle.north <= baseImageryRectangle.south) {
                new_rectangle.north = baseImageryRectangle.south;
                new_rectangle.south = baseImageryRectangle.south;
            } else {
                new_rectangle.south = baseTerrainRectangle.south.max(baseImageryRectangle.south);
                new_rectangle.north = baseTerrainRectangle.north.min(baseImageryRectangle.north);
            }

            if (baseTerrainRectangle.west >= baseImageryRectangle.east) {
                new_rectangle.west = baseImageryRectangle.east;
                new_rectangle.east = baseImageryRectangle.east;
            } else if (baseTerrainRectangle.east <= baseImageryRectangle.west) {
                new_rectangle.west = baseImageryRectangle.west;
                new_rectangle.east = baseImageryRectangle.west;
            } else {
                new_rectangle.west = baseTerrainRectangle.west.max(baseImageryRectangle.west);
                new_rectangle.east = baseTerrainRectangle.east.min(baseImageryRectangle.east);
            }
            intersection_rectangle = Some(new_rectangle)
        }

        let mut latitudeClosestToEquator = 0.0;
        if (rectangle.south > 0.0) {
            latitudeClosestToEquator = rectangle.south;
        } else if (rectangle.north < 0.0) {
            latitudeClosestToEquator = rectangle.north;
        }

        // Compute the required level in the imagery tiling scheme.
        // The errorRatio should really be imagerySSE / terrainSSE rather than this hard-coded value.
        // But first we need configurable imagery SSE and we need the rendering to be able to handle more
        // images attached to a terrain tile than there are available texture units.  So that's for the future.
        let errorRatio = 1.0;
        let targetGeometricError =
            errorRatio * terrain_datasource.getLevelMaximumGeometricError(key.level);
        let mut imageryLevel = self.getLevelWithMaximumTexelSpacing(
            targetGeometricError,
            latitudeClosestToEquator,
            &mut imagery_datasource,
        );
        imageryLevel = 0.max(imageryLevel);
        let maximumLevel = imagery_datasource.maximumLevel;
        if (imageryLevel > maximumLevel) {
            imageryLevel = maximumLevel;
        }
        let minimumLevel = imagery_datasource.minimumLevel;
        if (imageryLevel < minimumLevel) {
            imageryLevel = minimumLevel;
        }

        let imageryTilingScheme = imagery_datasource.tiling_scheme;
        let mut northwestTileCoordinates = imageryTilingScheme
            .position_to_tile_x_y(&rectangle.north_west(), imageryLevel)
            .expect("northwestTileCoordinates");
        let mut southeastTileCoordinates = imageryTilingScheme
            .position_to_tile_x_y(&rectangle.south_east(), imageryLevel)
            .expect("southeastTileCoordinates");

        // If the southeast corner of the rectangle lies very close to the north or west side
        // of the southeast tile, we don't actually need the southernmost or easternmost
        // tiles.
        // Similarly, if the northwest corner of the rectangle lies very close to the south or east side
        // of the northwest tile, we don't actually need the northernmost or westernmost tiles.

        // We define "very close" as being within 1/512 of the width of the tile.
        let mut veryCloseX = rectangle.computeWidth() / 512.0;
        let mut veryCloseY = rectangle.computeHeight() / 512.0;

        let northwestTileRectangle = imageryTilingScheme.tile_x_y_to_rectange(
            northwestTileCoordinates.x,
            northwestTileCoordinates.y,
            imageryLevel,
        );
        if (northwestTileRectangle.south - rectangle.north.abs() < veryCloseY
            && northwestTileCoordinates.y < southeastTileCoordinates.y)
        {
            northwestTileCoordinates.y += 1;
        }

        if ((northwestTileRectangle.east - rectangle.west).abs() < veryCloseX
            && northwestTileCoordinates.x < southeastTileCoordinates.x)
        {
            northwestTileCoordinates.x += 1;
        }

        let southeastTileRectangle = imageryTilingScheme.tile_x_y_to_rectange(
            southeastTileCoordinates.x,
            southeastTileCoordinates.y,
            imageryLevel,
        );
        if ((southeastTileRectangle.north - rectangle.south).abs() < veryCloseY
            && southeastTileCoordinates.y > northwestTileCoordinates.y)
        {
            southeastTileCoordinates.y -= 1;
        }
        if ((southeastTileRectangle.west - rectangle.east).abs() < veryCloseX
            && southeastTileCoordinates.x > northwestTileCoordinates.x)
        {
            southeastTileCoordinates.x -= 1;
        }

        // Create TileImagery instances for each imagery tile overlapping this terrain tile.
        // We need to do all texture coordinate computations in the imagery tile's tiling scheme.

        let terrainRectangle = rectangle.clone();
        let mut imageryRectangle = imageryTilingScheme.tile_x_y_to_rectange(
            northwestTileCoordinates.x,
            northwestTileCoordinates.y,
            imageryLevel,
        );
        let mut clippedImageryRectangle = imageryRectangle
            .intersection(&imageryBounds)
            .expect("clippedImageryRectangle");

        // let imageryTileXYToRectangle;
        let mut use_native = false;
        if (useWebMercatorT) {
            imageryTilingScheme.rectangle_to_native_rectangle(&terrainRectangle);
            imageryTilingScheme.rectangle_to_native_rectangle(&imageryRectangle);
            imageryTilingScheme.rectangle_to_native_rectangle(&clippedImageryRectangle);
            imageryTilingScheme.rectangle_to_native_rectangle(&imageryBounds);
            // imageryTileXYToRectangle = imageryTilingScheme
            //     .tile_x_y_to_native_rectange
            //     .bind(imageryTilingScheme);
            use_native = true;
            veryCloseX = terrainRectangle.computeWidth() / 512.0;
            veryCloseY = terrainRectangle.computeHeight() / 512.0;
        } else {
            // imageryTileXYToRectangle = imageryTilingScheme
            //     .tile_x_y_to_rectange
            //     .bind(imageryTilingScheme);
            use_native = false;
        }

        let mut minU;
        let mut maxU = 0.0;

        let mut minV = 1.0;
        let mut maxV;

        // If this is the northern-most or western-most tile in the imagery tiling scheme,
        // it may not start at the northern or western edge of the terrain tile.
        // Calculate where it does start.
        if (!self.isBaseLayer()
            && (clippedImageryRectangle.west - terrainRectangle.west).abs() >= veryCloseX)
        {
            maxU = (1.0 as f64).min(
                (clippedImageryRectangle.west - terrainRectangle.west)
                    / terrainRectangle.computeWidth(),
            );
        }

        if (!self.isBaseLayer()
            && (clippedImageryRectangle.north - terrainRectangle.north).abs() >= veryCloseY)
        {
            minV = (0.0 as f64).max(
                (clippedImageryRectangle.north - terrainRectangle.south)
                    / terrainRectangle.computeHeight(),
            );
        }

        let initialMinV = minV;
        for i in northwestTileCoordinates.x..southeastTileCoordinates.x {
            minU = maxU;

            imageryRectangle = if use_native {
                imagery_datasource
                    .tiling_scheme
                    .tile_x_y_to_native_rectange(i, northwestTileCoordinates.y, imageryLevel)
            } else {
                imagery_datasource.tiling_scheme.tile_x_y_to_rectange(
                    i,
                    northwestTileCoordinates.y,
                    imageryLevel,
                )
            };

            let clippedImageryRectangleRes = imageryRectangle.simpleIntersection(&imageryBounds);

            if (clippedImageryRectangleRes.is_none()) {
                continue;
            }
            clippedImageryRectangle = clippedImageryRectangleRes.expect("rectangle is some");

            maxU = (1.0 as f64).min(
                (clippedImageryRectangle.east - terrainRectangle.west)
                    / terrainRectangle.computeWidth(),
            );

            // If this is the eastern-most imagery tile mapped to this terrain tile,
            // and there are more imagery tiles to the east of this one, the maxU
            // should be 1.0 to make sure rounding errors don't make the last
            // image fall shy of the edge of the terrain tile.
            if (i == southeastTileCoordinates.x
                && (self.isBaseLayer()
                    || (clippedImageryRectangle.east - terrainRectangle.east).abs() < veryCloseX))
            {
                maxU = 1.0;
            }

            minV = initialMinV;
            for j in northwestTileCoordinates.y..southeastTileCoordinates.y {
                maxV = minV;

                imageryRectangle = if use_native {
                    imagery_datasource
                        .tiling_scheme
                        .tile_x_y_to_native_rectange(i, j, imageryLevel)
                } else {
                    imagery_datasource
                        .tiling_scheme
                        .tile_x_y_to_rectange(i, j, imageryLevel)
                };
                let clippedImageryRectangleRes =
                    imageryRectangle.simpleIntersection(&imageryBounds);

                if (clippedImageryRectangleRes.is_none()) {
                    continue;
                }
                clippedImageryRectangle = clippedImageryRectangleRes.expect("rectangle is some");
                minV = (0.0 as f64).max(
                    (clippedImageryRectangle.south - terrainRectangle.south)
                        / terrainRectangle.computeHeight(),
                );

                // If this is the southern-most imagery tile mapped to this terrain tile,
                // and there are more imagery tiles to the south of this one, the minV
                // should be 0.0 to make sure rounding errors don't make the last
                // image fall shy of the edge of the terrain tile.
                if (j == southeastTileCoordinates.y
                    && (self.isBaseLayer()
                        || (clippedImageryRectangle.south - terrainRectangle.south).abs()
                            < veryCloseY))
                {
                    minV = 0.0;
                }

                let texCoordsRectangle = DVec4::new(minU, minV, maxU, maxV);
                let imagery = self.getImageryFromCache(
                    &TileKey::new(i, j, imageryLevel),
                    None,
                    imagery_layer_entity.clone(),
                );
                globe_surface_tile.imagery.insert(
                    insertionPoint,
                    TileImagery::new(imagery.unwrap(), Some(texCoordsRectangle), useWebMercatorT),
                );
                insertionPoint += 1;
            }
        }

        return true;
    }
    pub fn getImageryFromCache(
        &mut self,
        key: &TileKey,
        imageryRectangle: Option<Rectangle>,
        imageryLayer: Entity,
    ) -> Option<Arc<Imagery>> {
        let id = key.get_id();
        let imagery = self._imageryCache.get(&id);
        if imagery.is_none() {
            let parent_key = key.parent();
            if let Some(parent_key) = key.parent() {
                let parent = self
                    ._imageryCache
                    .get(&parent_key.get_id())
                    .and_then(|x| Some(x.clone()));
                let new_imagery = Arc::new(Imagery::new(key.clone(), imageryLayer, parent));
                self._imageryCache.insert(id, new_imagery);
                return Some(new_imagery.clone());
            } else {
                return None;
            }
        } else {
            return Some(imagery.unwrap().clone());
        }
    }
    pub fn _calculateTextureTranslationAndScale(
        &self,
        tileImagery: &TileImagery,
        quand_tile_rectangle: &Rectangle,
        imagery_datasource_tiling_scheme: &GeographicTilingScheme,
    ) -> DVec4 {
        let mut imageryRectangle = tileImagery.readyImagery.unwrap().rectangle.clone();
        let mut quand_tile_rectangle = quand_tile_rectangle.clone();

        if (tileImagery.useWebMercatorT) {
            let tilingScheme = imagery_datasource_tiling_scheme;
            imageryRectangle = tilingScheme.rectangle_to_native_rectangle(&imageryRectangle);
            quand_tile_rectangle =
                tilingScheme.rectangle_to_native_rectangle(&quand_tile_rectangle);
        }

        let terrainWidth = quand_tile_rectangle.computeWidth();
        let terrainHeight = quand_tile_rectangle.computeHeight();

        let scaleX = terrainWidth / imageryRectangle.computeWidth();
        let scaleY = terrainHeight / imageryRectangle.computeHeight();
        return DVec4::new(
            (scaleX * (quand_tile_rectangle.west - imageryRectangle.west)) / terrainWidth,
            (scaleY * (quand_tile_rectangle.south - imageryRectangle.south)) / terrainHeight,
            scaleX,
            scaleY,
        );
    }
    pub fn _reprojectTexture(&self, imagery: &Arc<Imagery>, needGeographicProjection: bool) {}
}
#[derive(Component)]
pub struct XYZDataSource {
    pub ready: bool,
    pub rectangle: Rectangle,
    pub tiling_scheme: GeographicTilingScheme,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimumLevel: u32,
    pub maximumLevel: u32,
}
impl Default for XYZDataSource {
    fn default() -> Self {
        Self {
            ready: true,
            rectangle: Rectangle::MAX_VALUE.clone(),
            tiling_scheme: GeographicTilingScheme::default(),
            tile_height: 256,
            tile_width: 256,
            minimumLevel: 0,
            maximumLevel: 31,
        }
    }
}
impl XYZDataSource {
    pub fn requestImage(
        &self,
        key: &TileKey,
        asset_server: &Res<AssetServer>,
    ) -> Option<Handle<Image>> {
        return Some(asset_server.load(format!(
            "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
            key.level, key.x, key.y,
        )));
    }
}

#[derive(Bundle)]
pub struct ImageryLayerBundle {
    visibility: Visibility,
    layer: ImageryLayer,
    datasource: XYZDataSource,
}
impl ImageryLayerBundle {
    pub fn new(imagery_layer_entity: Entity) -> Self {
        Self {
            visibility: Visibility::Visible,
            layer: ImageryLayer::new(imagery_layer_entity),
            datasource: XYZDataSource::default(),
        }
    }
    pub fn spawn(commands: &mut Commands) {}
}
#[derive(Component)]
pub struct TerrainDataSource {
    pub tiling_scheme: Arc<GeographicTilingScheme>,
    _levelZeroMaximumGeometricError: f64,
    pub ready: bool,
    pub rectangle: Rectangle,
}
impl TerrainDataSource {
    pub fn new() -> Self {
        let tiling_scheme = GeographicTilingScheme::default();
        let _levelZeroMaximumGeometricError = get_levelZeroMaximumGeometricError(&tiling_scheme);
        let tiling_scheme_arc = Arc::new(tiling_scheme);
        Self {
            tiling_scheme: tiling_scheme_arc,
            _levelZeroMaximumGeometricError: _levelZeroMaximumGeometricError,
            ready: true,
            rectangle: Rectangle::MAX_VALUE.clone(),
        }
    }
    pub fn getTileDataAvailable(&self, key: &TileKey) -> bool {
        return false;
    }
    pub fn loadTileDataAvailability(&self, key: &TileKey) -> bool {
        return false;
    }
    pub fn getLevelMaximumGeometricError(&self, level: u32) -> f64 {
        return self._levelZeroMaximumGeometricError / (1 << level) as f64;
    }

    pub async fn requestTileGeometry(&self, key: &TileKey) -> Option<HeightmapTerrainData> {
        let width = 16;
        let height = 16;
        return Some(HeightmapTerrainData::new(
            vec![0.; width * height],
            width as u32,
            height as u32,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }
}
fn getEstimatedLevelZeroGeometricErrorForAHeightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    numberOfTilesAtLevelZero: u32,
) -> f64 {
    return ((ellipsoid.maximumRadius * 2. * PI * 0.25)
        / (tile_image_width as f64 * numberOfTilesAtLevelZero as f64));
}
fn get_levelZeroMaximumGeometricError(tiling_scheme: &GeographicTilingScheme) -> f64 {
    return getEstimatedLevelZeroGeometricErrorForAHeightmap(
        &tiling_scheme.ellipsoid,
        64,
        tiling_scheme.get_number_of_tiles_at_level(0),
    );
}
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
fn setup(mut commands: Commands) {
    //terrain datasource
    commands.spawn(TerrainDataSource::new());
    //测试的imagerylayer
    let mut entity_mut = commands.spawn_empty();
    let imagery_layer_entity = entity_mut.id();
    entity_mut.insert(ImageryLayerBundle::new(imagery_layer_entity));
}
