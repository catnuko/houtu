use std::{
    any::type_name,
    sync::{Arc, Mutex},
};

use crate::{
    getEstimatedLevelZeroGeometricErrorForAHeightmap,
    lerp,
    // getEstimatedLevelZeroGeometricErrorForAHeightmap, getRegularGridAndSkirtIndicesAndEdgeIndices,
    // getRegularGridIndicesAndEdgeIndices,
    CreateVerticeOptions,
    CreateVerticeReturn,
    GeographicTilingScheme,
    HeightmapEncoding,
    HeightmapTerrainStructure,
    IndicesAndEdgesCache,
    Rectangle,
    TerrainEncoding,
    TerrainMesh,
    TileKey,
    TilingScheme,
};

use super::create_vertice;
#[derive(Debug)]
pub struct HeightmapTerrainData {
    pub _buffer: Vec<f32>,
    pub _width: u32,
    pub _height: u32,
    pub _childTileMask: i32,
    pub _encoding: HeightmapEncoding,
    pub _structure: HeightmapTerrainStructure,
    pub _createdByUpsampling: bool,
    pub _waterMask: Option<Vec<u8>>,
    pub _skirtHeight: Option<f64>,
    pub _mesh: Option<TerrainMesh>,
}

impl HeightmapTerrainData {
    pub fn new(
        buffer: Vec<f32>,
        width: u32,
        height: u32,
        childTileMask: Option<i32>,
        encoding: Option<HeightmapEncoding>,
        structure: Option<HeightmapTerrainStructure>,
        createdByUpsampling: Option<bool>,
        waterMask: Option<Vec<u8>>,
        skirtHeight: Option<f64>,
        mesh: Option<TerrainMesh>,
    ) -> Self {
        Self {
            _buffer: buffer,
            _width: width,
            _height: height,
            _childTileMask: childTileMask.unwrap_or(15),
            _encoding: encoding.unwrap_or(HeightmapEncoding::NONE),
            _structure: structure.unwrap_or(HeightmapTerrainStructure::default()),
            _createdByUpsampling: createdByUpsampling.unwrap_or(false),
            _waterMask: waterMask,
            _skirtHeight: skirtHeight,
            _mesh: mesh,
        }
    }
    pub fn canUpsample(&self) -> bool {
        return self._mesh.is_some();
    }
    pub fn isChildAvailable(&self, thisX: u32, thisY: u32, childX: u32, childY: u32) -> bool {
        let mut bitNumber = 2; // northwest child
        if (childX != thisX * 2) {
            bitNumber += 1; // east child
        }
        if (childY != thisY * 2) {
            bitNumber -= 2; // south child
        }

        return (self._childTileMask & (1 << bitNumber)) != 0;
    }
    pub fn wasCreatedByUpsampling(&self) -> bool {
        return self._createdByUpsampling;
    }
    pub async fn createMesh<T: TilingScheme>(
        &mut self,
        tilingScheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggerationRelativeHeight: Option<f64>,
        indicesAndEdgesCacheArc: Arc<Mutex<IndicesAndEdgesCache>>,
    ) {
        let result = self.create_vertice(
            tilingScheme,
            x,
            y,
            level,
            exaggeration,
            exaggerationRelativeHeight,
        );

        let mut indicesAndEdgesCache = indicesAndEdgesCacheArc.lock().unwrap();
        let indicesAndEdges;
        if (self._skirtHeight.unwrap() > 0.0) {
            indicesAndEdges = indicesAndEdgesCache
                .getRegularGridAndSkirtIndicesAndEdgeIndices(self._width, self._height);
        } else {
            indicesAndEdges =
                indicesAndEdgesCache.getRegularGridIndicesAndEdgeIndices(self._width, self._height);
        }

        let vertexCountWithoutSkirts = 0;
        self._mesh = Some(TerrainMesh::new(
            result.relativeToCenter.unwrap(),
            result.vertices,
            indicesAndEdges.indices,
            indicesAndEdges.indexCountWithoutSkirts,
            vertexCountWithoutSkirts,
            result.minimumHeight,
            result.maximumHeight,
            result.boundingSphere3D,
            result.occludeePointInScaledSpace,
            result.encoding.stride,
            result.orientedBoundingBox,
            result.encoding,
            indicesAndEdges.westIndicesSouthToNorth,
            indicesAndEdges.southIndicesEastToWest,
            indicesAndEdges.eastIndicesNorthToSouth,
            indicesAndEdges.northIndicesWestToEast,
        ));
    }

    pub fn create_vertice<T: TilingScheme>(
        &mut self,
        tilingScheme: &T,
        x: u32,
        y: u32,
        level: u32,
        exaggeration: Option<f64>,
        exaggerationRelativeHeight: Option<f64>,
    ) -> CreateVerticeReturn {
        let tilingScheme = tilingScheme;
        let x = x;
        let y = y;
        let level = level;
        let exaggeration = exaggeration.unwrap_or(1.0);
        let exaggerationRelativeHeight = exaggerationRelativeHeight.unwrap_or(0.0);

        let ellipsoid = tilingScheme.get_ellipsoid();
        let nativeRectangle = tilingScheme.tile_x_y_to_native_rectange(x, y, level);
        let rectangle = tilingScheme.tile_x_y_to_rectange(x, y, level);

        // Compute the center of the tile for RTC rendering.
        let center = ellipsoid.cartographicToCartesian(&rectangle.center());

        let structure = self._structure;

        let levelZeroMaxError = getEstimatedLevelZeroGeometricErrorForAHeightmap(
            &ellipsoid,
            self._width,
            tilingScheme.get_number_of_x_tiles_at_level(0),
        );
        let thisLevelMaxError = levelZeroMaxError / (1 << level) as f64;
        let skirtHeight = (thisLevelMaxError * 4.0).min(1000.0);
        self._skirtHeight = Some(skirtHeight);
        let result = create_vertice(CreateVerticeOptions {
            heightmap: &mut self._buffer,
            structure: Some(structure),
            includeWebMercatorT: Some(true),
            width: self._width,
            height: self._height,
            nativeRectangle: nativeRectangle,
            rectangle: Some(rectangle),
            relativeToCenter: Some(center),
            ellipsoid: Some(ellipsoid),
            skirtHeight: skirtHeight,
            isGeographic: Some(
                type_name::<T>() == "houtu_scene::geographic_tiling_scheme::GeographicTilingScheme",
            ),
            exaggeration: Some(exaggeration),
            exaggerationRelativeHeight: Some(exaggerationRelativeHeight),
        });
        return result;
    }
    pub async fn upsample(
        &self,
        tiling_scheme: &GeographicTilingScheme,
        thisX: u32,
        thisY: u32,
        thisLevel: u32,
        descendantX: u32,
        descendantY: u32,
        descendantLevel: u32,
    ) -> Option<HeightmapTerrainData> {
        if self._mesh.is_none() {
            return None;
        }
        let meshData = self._mesh.as_ref().unwrap();

        let width = self._width;
        let height = self._height;
        let structure = self._structure;
        let stride = structure.stride;

        let mut heights: Vec<f32> = vec![0.; (width * height * stride) as usize];

        let buffer = &meshData.vertices;
        let encoding = meshData.encoding;

        // PERFORMANCE_IDEA: don't recompute these rectangles - the caller already knows them.
        let sourceRectangle = tiling_scheme.tile_x_y_to_rectange(thisX, thisY, thisLevel);
        let destinationRectangle =
            tiling_scheme.tile_x_y_to_rectange(descendantX, descendantY, descendantLevel);

        let heightOffset = structure.heightOffset;
        let heightScale = structure.heightScale;

        let elementsPerHeight = structure.elementsPerHeight;
        let elementMultiplier = structure.elementMultiplier;
        let isBigEndian = structure.isBigEndian;

        let divisor = elementMultiplier.pow(elementsPerHeight - 1);

        for j in 0..height {
            let latitude = lerp(
                destinationRectangle.north,
                destinationRectangle.south,
                (j / (height - 1)) as f64,
            );
            for i in 0..width {
                let longitude = lerp(
                    destinationRectangle.west,
                    destinationRectangle.east,
                    (i / (width - 1)) as f64,
                );
                let mut heightSample = interpolateMeshHeight(
                    &buffer,
                    &encoding,
                    heightOffset,
                    heightScale,
                    &sourceRectangle,
                    width,
                    height,
                    longitude,
                    latitude,
                );

                // Use conditionals here instead of Math.min and Math.max so that an undefined
                // lowestEncodedHeight or highestEncodedHeight has no effect.
                heightSample = if heightSample < structure.lowestEncodedHeight {
                    structure.lowestEncodedHeight
                } else {
                    heightSample
                };
                heightSample = if heightSample > structure.highestEncodedHeight {
                    structure.highestEncodedHeight
                } else {
                    heightSample
                };

                setHeight(
                    &mut heights,
                    elementsPerHeight,
                    elementMultiplier,
                    divisor,
                    stride,
                    isBigEndian,
                    j * width + i,
                    heightSample,
                );
            }
        }
        return Some(HeightmapTerrainData::new(
            heights,
            width,
            height,
            Some(0),
            None,
            Some(self._structure.clone()),
            Some(true),
            None,
            None,
            None,
        ));
    }
}

fn interpolateMeshHeight(
    buffer: &Vec<f32>,
    encoding: &TerrainEncoding,
    heightOffset: f64,
    heightScale: f64,
    sourceRectangle: &Rectangle,
    width: u32,
    height: u32,
    longitude: f64,
    latitude: f64,
) -> f64 {
    // returns a height encoded according to the structure's heightScale and heightOffset.
    let fromWest = ((longitude - sourceRectangle.west) * (width - 1) as f64)
        / (sourceRectangle.east - sourceRectangle.west);
    let fromSouth = ((latitude - sourceRectangle.south) * (height - 1) as f64)
        / (sourceRectangle.north - sourceRectangle.south);

    let mut westInteger = fromWest.round() as u32;
    let mut eastInteger = westInteger + 1;
    if (eastInteger >= width) {
        eastInteger = width - 1;
        westInteger = width - 2;
    }

    let mut southInteger = fromSouth.round() as u32;
    let mut northInteger = southInteger + 1;
    if (northInteger >= height) {
        northInteger = height - 1;
        southInteger = height - 2;
    }

    let dx = fromWest - westInteger as f64;
    let dy = fromSouth - southInteger as f64;

    southInteger = height - 1 - southInteger;
    northInteger = height - 1 - northInteger;

    let southwestHeight = (encoding
        .decodeHeight(buffer, (southInteger * width + westInteger) as usize)
        - heightOffset)
        / heightScale;
    let southeastHeight = (encoding
        .decodeHeight(buffer, (southInteger * width + eastInteger) as usize)
        - heightOffset)
        / heightScale;
    let northwestHeight = (encoding
        .decodeHeight(buffer, (northInteger * width + westInteger) as usize)
        - heightOffset)
        / heightScale;
    let northeastHeight = (encoding
        .decodeHeight(buffer, (northInteger * width + eastInteger) as usize)
        - heightOffset)
        / heightScale;

    return triangleInterpolateHeight(
        dx,
        dy,
        southwestHeight,
        southeastHeight,
        northwestHeight,
        northeastHeight,
    );
}

fn triangleInterpolateHeight(
    dX: f64,
    dY: f64,
    southwestHeight: f64,
    southeastHeight: f64,
    northwestHeight: f64,
    northeastHeight: f64,
) -> f64 {
    // The HeightmapTessellator bisects the quad from southwest to northeast.
    if (dY < dX) {
        // Lower right triangle
        return (southwestHeight
            + dX * (southeastHeight - southwestHeight)
            + dY * (northeastHeight - southeastHeight));
    }

    // Upper left triangle
    return (southwestHeight
        + dX * (northeastHeight - northwestHeight)
        + dY * (northwestHeight - southwestHeight));
}
fn setHeight(
    heights: &mut Vec<f32>,
    elementsPerHeight: u32,
    elementMultiplier: u32,
    divisor: u32,
    stride: u32,
    isBigEndian: bool,
    index: u32,
    height: f64,
) {
    let mut height = height as f32;
    let mut divisor = divisor;
    let index = index * stride;
    let mut j = 0;
    if (isBigEndian) {
        for i in 0..elementsPerHeight - 1 {
            heights[(index + i) as usize] = (height / divisor as f32).round();
            height -= heights[(index + i) as usize] * divisor as f32;
            divisor /= elementMultiplier;
        }
        j = elementsPerHeight - 2;
    } else {
        for i in (0..elementsPerHeight - 1).rev() {
            heights[(index + i) as usize] = (height / divisor as f32).round();
            height -= heights[(index + i) as usize] * divisor as f32;
            divisor /= elementMultiplier;
        }
        j = 1;
    }
    heights[(index + j) as usize] = height;
}
//   fn getHeight(
//     heights,
//     elementsPerHeight,
//     elementMultiplier,
//     stride,
//     isBigEndian,
//     index
//   ) {
//     index *= stride;

//     let height = 0;
//     let i;

//     if (isBigEndian) {
//       for (i = 0; i < elementsPerHeight; ++i) {
//         height = height * elementMultiplier + heights[index + i];
//       }
//     } else {
//       for (i = elementsPerHeight - 1; i >= 0; --i) {
//         height = height * elementMultiplier + heights[index + i];
//       }
//     }

//     return height;
//   }
