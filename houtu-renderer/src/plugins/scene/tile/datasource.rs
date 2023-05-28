use std::f64::consts::PI;

use bevy::math::DVec3;
use houtu_scene::{
    Ellipsoid, GeographicProjection, GeographicTilingScheme, Tile, TilingScheme, EPSILON5,
};

pub trait DataSource {
    fn get_tiling_scehme(&self) -> &dyn TilingScheme;
    fn is_ready(&self) -> bool;
    fn computeTileLoadPriority(
        &self,
        tile: &Tile,
        cameraPositionWC: &DVec3,
        cameraDirectionWC: &DVec3,
    ) -> f64;
    fn getLevelMaximumGeometricError(&self, level: u32) -> f64;
    fn canRenderWithoutLosingDetail(&self, tile: &Tile) -> bool;
}

#[derive(Default)]
pub struct GlobeSurfaceTileDataSource {
    pub tiling_scheme: GeographicTilingScheme,
    pub is_ready: bool,
    pub _levelZeroMaximumGeometricError: f64,
    pub heightmapTerrainQuality: f64,
}
impl GlobeSurfaceTileDataSource {
    pub fn new() -> Self {
        let tiling_scheme = GeographicTilingScheme::default();
        let _levelZeroMaximumGeometricError = getEstimatedLevelZeroGeometricErrorForAHeightmap(
            &tiling_scheme,
            64,
            tiling_scheme.get_number_of_x_tiles_at_level(0),
        );
        Self {
            tiling_scheme: tiling_scheme,
            is_ready: false,
            heightmapTerrainQuality: 0.25,
            _levelZeroMaximumGeometricError,
        }
    }
}
impl DataSource for GlobeSurfaceTileDataSource {
    fn get_tiling_scehme(&self) -> &dyn TilingScheme {
        return self.tiling_scheme as &dyn TilingScheme;
    }
    fn is_ready(&self) -> bool {
        return self.is_ready;
    }
    fn computeTileLoadPriority(
        &self,
        tile: &Tile,
        cameraPositionWC: &DVec3,
        cameraDirectionWC: &DVec3,
    ) -> f64 {
        let obb = tile.tileBoundingRegion.orientedBoundingBox;
        if (obb.is_none()) {
            return 0.0;
        }
        let mut tileDirection = obb.unwrap().center.subtract(*cameraPositionWC);
        let magnitude = tileDirection.magnitude();
        if (magnitude < EPSILON5) {
            return 0.0;
        }
        tileDirection = tileDirection.divide_by_scalar(magnitude);
        return ((1.0 - tileDirection.dot(*cameraDirectionWC)) * tile._distance);
    }
    fn getLevelMaximumGeometricError(&self, level: u32) -> f64 {
        return self._levelZeroMaximumGeometricError / (1 << level);
    }
    //     fn canRenderWithoutLosingDetail(&self, tile: &Tile) -> bool {
    //         let surfaceTile = tile.data;

    //         let readyImagery = vec![];
    //         readyImagery.length = this._imageryLayers.length;

    //         let terrainReady = false;

    //         let initialImageryState = false;
    //         let imagery;

    //         if (defined(surfaceTile)) {
    //           // We can render even with non-ready terrain as long as all our rendered descendants
    //           // are missing terrain geometry too. i.e. if we rendered fills for more detailed tiles
    //           // last frame, it's ok to render a fill for this tile this frame.
    //           terrainReady = surfaceTile.terrainState == TerrainState.READY;

    //           // Initially assume all imagery layers are ready, unless imagery hasn't been initialized at all.
    //           initialImageryState = true;

    //           imagery = surfaceTile.imagery;
    //         }

    //         let i;
    //         let len;
    //         for i in 0..readyImagery.len(){
    //           readyImagery[i] = initialImageryState;
    //         }

    //         if (defined(imagery)) {
    //             for i in 0..imagery.len(){
    //             let tileImagery = imagery[i];
    //             let loadingImagery = tileImagery.loadingImagery;
    //             let isReady =
    //               !defined(loadingImagery) ||
    //               loadingImagery.state == ImageryState.FAILED ||
    //               loadingImagery.state == ImageryState.INVALID;
    //             let layerIndex = (
    //               tileImagery.loadingImagery || tileImagery.readyImagery
    //             ).imageryLayer._layerIndex;

    //             // For a layer to be ready, all tiles belonging to that layer must be ready.
    //             readyImagery[layerIndex] = isReady && readyImagery[layerIndex];
    //           }
    //         }

    //         let lastFrame = this.quadtree._lastSelectionFrameNumber;

    //         // Traverse the descendants looking for one with terrain or imagery that is not loaded on this tile.
    //         let stack = vec![];
    //         stack.length = 0;
    //         stack.push(
    //             tile.southwestChild);
    //         stack.push(
    //             tile.southeastChild);
    //         stack.push(
    //             tile.northwestChild);
    //         stack.push(
    //             tile.northeastChild);

    //         while (stack.length > 0) {
    //           let descendant = stack.pop();
    //           let lastFrameSelectionResult =
    //             descendant._lastSelectionResultFrame == lastFrame
    //               ? descendant._lastSelectionResult
    //               : TileSelectionResult::NONE;

    //           if (lastFrameSelectionResult == TileSelectionResult::RENDERED) {
    //             let descendantSurface = descendant.data;

    //             if (!defined(descendantSurface)) {
    //               // Descendant has no data, so it can't block rendering.
    //               continue;
    //             }

    //             if (
    //               !terrainReady &&
    //               descendant.data.terrainState == TerrainState.READY
    //             ) {
    //               // Rendered descendant has real terrain, but we don't. Rendering is blocked.
    //               return false;
    //             }

    //             let descendantImagery = descendant.data.imagery;
    //             for i in 0..descendantImagery.len(){
    //               let descendantTileImagery = descendantImagery[i];
    //               let descendantLoadingImagery = descendantTileImagery.loadingImagery;
    //               let descendantIsReady =
    //                 !defined(descendantLoadingImagery) ||
    //                 descendantLoadingImagery.state == ImageryState.FAILED ||
    //                 descendantLoadingImagery.state == ImageryState.INVALID;
    //               let descendantLayerIndex = (
    //                 descendantTileImagery.loadingImagery ||
    //                 descendantTileImagery.readyImagery
    //               ).imageryLayer._layerIndex;

    //               // If this imagery tile of a descendant is ready but the layer isn't ready in this tile,
    //               // then rendering is blocked.
    //               if (descendantIsReady && !readyImagery[descendantLayerIndex]) {
    //                 return false;
    //               }
    //             }
    //           } else if (lastFrameSelectionResult == TileSelectionResult::REFINED) {
    //             stack.push(
    //                 descendant.southwestChild,);
    //             stack.push(
    //                 descendant.southeastChild,);
    //             stack.push(
    //                 descendant.northwestChild,);
    //             stack.push(
    //                 descendant.southwestChild,);
    //           }
    //         }

    //         return true;
    //     }
}
fn getEstimatedLevelZeroGeometricErrorForAHeightmap(
    ellipsoid: &Ellipsoid,
    tileImageWidth: u32,
    numberOfTilesAtLevelZero: u32,
) -> f64 {
    let heightmapTerrainQuality = 0.25;
    return ((ellipsoid.maximumRadius * 2 * PI * heightmapTerrainQuality)
        / (tileImageWidth * numberOfTilesAtLevelZero));
}
