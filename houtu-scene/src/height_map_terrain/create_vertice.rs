#![warn(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::expect_used
)]
use crate::{
    ellipsoidal_occluder::EllipsoidalOccluder,
    geometry::{AxisAlignedBoundingBox, BoundingSphere, OrientedBoundingBox},
    math::{eastNorthUpToFixedFrame, Cartesian3, Matrix4},
    terrain_encoding::TerrainEncoding,
    web_mercator_projection::WebMercatorProjection,
};

use super::*;
use bevy::{
    math::{DVec2, DVec3},
    render::primitives::Aabb,
};
use std::{
    f64::{consts::FRAC_PI_2, MAX, MIN_POSITIVE},
    io,
};
pub struct CreateMeshJobOutput {
    pub vertices: Vec<f64>,
    pub maximumHeight: f64,
    pub minimumHeight: f64,
    pub encoding: f64,
    pub boundingSphere3D: f64,
    pub orientedBoundingBox: f64,
    pub occludeePointInScaledSpace: f64,
}

// pub struct CreateMeshJob {
//     pub url: String,
//     pub crs: String,
//     pub name: String,
// }

// #[derive(thiserror::Error, Debug)]
// pub enum Error {
//     #[error("{0}")]
//     Io(#[from] io::Error),
//     #[error("{0}")]
//     Reqwest(#[from] reqwest::Error),
// }

// impl bevy_jobs::Job for CreateMeshJob {
//     type Outcome = Result<CreateMeshJobOutput, Error>;

//     fn name(&self) -> String {
//         format!("Fetching '{}'", self.name)
//     }

//     fn perform(self, ctx: bevy_jobs::Context) -> bevy_jobs::AsyncReturn<Self::Outcome> {
//         Box::pin(async move {
//             let fetch = async {
//                 // Ok(FetchedFile {
//                 //     bytes: bytes::Bytes::from(bytes),
//                 //     crs: self.crs,
//                 //     name: self.name,
//                 // })
//                 None
//             };
//             #[cfg(not(target_arch = "wasm32"))]
//             {
//                 let runtime = tokio::runtime::Builder::new_current_thread()
//                     .enable_all()
//                     .build()?;
//                 runtime.block_on(fetch)
//             }
//             #[cfg(target_arch = "wasm32")]
//             {
//                 fetch.await
//             }
//         })
//     }
// }

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}
#[derive(Debug, Clone, PartialEq)]
pub struct CreateVerticeOptions {
    pub heightmap: Vec<f64>,
    pub width: u32,
    pub height: u32,
    pub skirtHeight: f64,
    pub nativeRectangle: Rectangle,
    pub exaggeration: Option<f64>,
    pub exaggerationRelativeHeight: Option<f64>,
    pub rectangle: Option<Rectangle>,
    pub isGeographic: Option<bool>,
    pub relativeToCenter: Option<DVec3>,
    pub ellipsoid: Option<Ellipsoid>,
    pub structure: Option<HeightmapTerrainStructure>,
    pub includeWebMercatorT: Option<bool>,
}
pub struct CreateVerticeReturn {
    pub vertices: Vec<f64>,
    pub maximumHeight: f64,
    pub minimumHeight: f64,
    pub encoding: TerrainEncoding,
    pub boundingSphere3D: BoundingSphere,
    pub orientedBoundingBox: OrientedBoundingBox,
    pub occludeePointInScaledSpace: DVec3,
}
pub fn create_vertice(options: CreateVerticeOptions) -> CreateVerticeReturn {
    let piOverTwo = FRAC_PI_2;
    let heightmap = options.heightmap;
    let width = options.width;
    let height = options.height;
    let skirtHeight = options.skirtHeight;
    let hasSkirts = skirtHeight > 0.0;
    let isGeographic = options.isGeographic.unwrap_or(true);
    let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

    let oneOverGlobeSemimajorAxis = 1.0 / ellipsoid.maximumRadius;

    let nativeRectangle = options.nativeRectangle.clone();
    let rectangleOption = options.rectangle;

    let mut geographicWest: f64;
    let mut geographicSouth: f64;
    let mut geographicEast: f64;
    let mut geographicNorth: f64;
    if let Some(rectangle) = rectangleOption {
        geographicWest = rectangle.west;
        geographicSouth = rectangle.south;
        geographicEast = rectangle.east;
        geographicNorth = rectangle.north;
    } else {
        if isGeographic {
            geographicWest = nativeRectangle.west.to_radians();
            geographicSouth = nativeRectangle.south.to_radians();
            geographicEast = nativeRectangle.east.to_radians();
            geographicNorth = nativeRectangle.north.to_radians();
        } else {
            geographicWest = nativeRectangle.west * oneOverGlobeSemimajorAxis;
            // geographicSouth =
            //     piOverTwo - 2.0 * atan(exp(-nativeRectangle.south * oneOverGlobeSemimajorAxis));
            geographicSouth = piOverTwo
                - 2.0
                    * (-nativeRectangle.south * oneOverGlobeSemimajorAxis)
                        .exp()
                        .atan();
            geographicEast = nativeRectangle.east * oneOverGlobeSemimajorAxis;
            geographicNorth = piOverTwo
                - 2.0
                    * (-nativeRectangle.north * oneOverGlobeSemimajorAxis)
                        .exp()
                        .atan();
        }
    }
    let mut relativeToCenter = options.relativeToCenter.unwrap_or(DVec3::ZERO);
    let hasRelativeToCenter = options.relativeToCenter.is_some();
    let includeWebMercatorT = options.includeWebMercatorT.unwrap_or(false);
    let exaggeration = options.exaggeration.unwrap_or(1.0);
    let exaggerationRelativeHeight = options.exaggerationRelativeHeight.unwrap_or(0.0);
    let hasExaggeration = exaggeration != 1.0;
    let includeGeodeticSurfaceNormals = hasExaggeration;
    let structore = options
        .structure
        .unwrap_or(HeightmapTerrainStructure::default());

    let heightScale = structore.heightScale;
    let heightOffset = structore.heightOffset;
    let elementsPerHeight = structore.elementsPerHeight;
    let stride = structore.stride;
    let elementMultiplier = structore.elementMultiplier;
    let isBigEndian = structore.isBigEndian;

    let mut rectangleWidth = nativeRectangle.computeWidth();
    let mut rectangleHeight = nativeRectangle.computeHeight();

    let granularityX = rectangleWidth / (width as f64 - 1.);
    let granularityY = rectangleHeight / (height as f64 - 1.);

    if (!isGeographic) {
        rectangleWidth *= oneOverGlobeSemimajorAxis;
        rectangleHeight *= oneOverGlobeSemimajorAxis;
    }

    let radiiSquared = ellipsoid.radiiSquared;
    let radiiSquaredX = radiiSquared.x;
    let radiiSquaredY = radiiSquared.y;
    let radiiSquaredZ = radiiSquared.z;

    let mut minimumHeight: f64 = 65536.0;
    let mut maximumHeight: f64 = -65536.0;

    let fromENU = eastNorthUpToFixedFrame(relativeToCenter, Some(ellipsoid));
    let toENU = fromENU.inverse_transformation();
    let webMercatorProjection = WebMercatorProjection::default();
    let mut southMercatorY = 0.;
    let mut oneOverMercatorHeight = 0.;
    if (includeWebMercatorT) {
        southMercatorY = webMercatorProjection.geodeticLatitude_to_mercator_angle(geographicSouth);
        oneOverMercatorHeight = 1.0
            / (webMercatorProjection.geodeticLatitude_to_mercator_angle(geographicNorth)
                - southMercatorY);
    }

    let mut minimum = DVec3::ZERO;
    minimum.x = MAX;
    minimum.y = MAX;
    minimum.z = MAX;

    let mut maximum = DVec3::ZERO;
    maximum.x = MIN_POSITIVE;
    maximum.y = MIN_POSITIVE;
    maximum.z = MIN_POSITIVE;

    let mut hMin = MAX;

    let gridVertexCount: u32 = width * height;
    let edgeVertexCount: u32 = {
        if skirtHeight > 0.0 {
            width * 2 + height * 2
        } else {
            0
        }
    };

    let vertexCount = gridVertexCount + edgeVertexCount;
    let mut positions: Vec<DVec3> = Vec::with_capacity(vertexCount as usize); // 预分配内存空间
    positions.extend(std::iter::repeat(DVec3::ZERO).take(vertexCount as usize));
    let mut heights: Vec<f64> = Vec::with_capacity(vertexCount as usize); // 预分配内存空间
    heights.extend(std::iter::repeat(0.).take(vertexCount as usize));
    let mut uvs: Vec<DVec2> = Vec::with_capacity(vertexCount as usize); // 预分配内存空间
    uvs.extend(std::iter::repeat(DVec2::ZERO).take(vertexCount as usize));

    let mut webMercatorTsOption = {
        if includeWebMercatorT {
            let mut tmp = Vec::with_capacity(vertexCount as usize); // 预分配内存空间
            tmp.extend(std::iter::repeat(0.).take(vertexCount as usize));
            Some(tmp)
        } else {
            None
        }
    };
    let mut geodeticSurfaceNormalsOption = {
        if includeGeodeticSurfaceNormals {
            let mut tmp: Vec<DVec3> = Vec::with_capacity(vertexCount as usize); // 预分配内存空间
            tmp.extend(std::iter::repeat(DVec3::ZERO).take(vertexCount as usize));
            Some(tmp)
        } else {
            None
        }
    };
    let mut startRow = 0;
    let mut endRow = height;
    let mut startCol = 0;
    let mut endCol = width;

    if (hasSkirts) {
        startRow -= 1;
        endRow += 1;
        startCol -= 1;
        endCol += 1;
    }

    let skirtOffsetPercentage = 0.00001;
    for rowIndex in startRow..endRow {
        let mut row = rowIndex;
        if (row < 0) {
            row = 0;
        }
        if (row >= height) {
            row = height - 1;
        }

        let mut latitude = nativeRectangle.north - granularityY * row as f64;

        if (!isGeographic) {
            latitude = piOverTwo - 2.0 * ((-latitude * oneOverGlobeSemimajorAxis).exp()).atan();
        } else {
            latitude = (latitude).to_radians();
        }

        let mut v = (latitude - geographicSouth) / (geographicNorth - geographicSouth);
        v = v.clamp(0.0, 1.0);

        let isNorthEdge = rowIndex == startRow;
        let isSouthEdge = rowIndex == endRow - 1;
        if (skirtHeight > 0.0) {
            if (isNorthEdge) {
                latitude += skirtOffsetPercentage * rectangleHeight;
            } else if (isSouthEdge) {
                latitude -= skirtOffsetPercentage * rectangleHeight;
            }
        }

        let cosLatitude = (latitude).cos();
        let nZ = (latitude).sin();
        let kZ = radiiSquaredZ * nZ;

        let mut webMercatorT: f64 = 0.;
        if (includeWebMercatorT) {
            webMercatorT = (webMercatorProjection.geodeticLatitude_to_mercator_angle(latitude)
                - southMercatorY)
                * oneOverMercatorHeight;
        }
        for colIndex in startCol..endCol {
            let mut col = colIndex;
            if (col < 0) {
                col = 0;
            }
            if (col >= width) {
                col = width - 1;
            }

            let terrainOffset = row * (width * stride) + col * stride;

            let mut heightSample: f64;
            if (elementsPerHeight == 1) {
                heightSample = heightmap[terrainOffset as usize];
            } else {
                heightSample = 0.;

                if (isBigEndian) {
                    for elementOffset in 0..elementsPerHeight {
                        heightSample = heightSample * elementMultiplier as f64
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                } else {
                    //可能会出问题，注意
                    for elementOffset in (0..elementsPerHeight).rev() {
                        heightSample = heightSample * elementMultiplier as f64
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                }
            }

            heightSample = heightSample * heightScale + heightOffset;

            maximumHeight = maximumHeight.max(heightSample);
            minimumHeight = minimumHeight.min(heightSample);

            let mut longitude = nativeRectangle.west + granularityX * (col as f64);

            if (!isGeographic) {
                longitude = longitude * oneOverGlobeSemimajorAxis;
            } else {
                longitude = (longitude).to_radians();
            }

            let mut u = (longitude - geographicWest) / (geographicEast - geographicWest);
            u = u.clamp(0.0, 1.0);

            let mut index = row * width + col;

            if (skirtHeight > 0.0) {
                let isWestEdge = colIndex == startCol;
                let isEastEdge = colIndex == endCol - 1;
                let isEdge = isNorthEdge || isSouthEdge || isWestEdge || isEastEdge;
                let isCorner = (isNorthEdge || isSouthEdge) && (isWestEdge || isEastEdge);
                if (isCorner) {
                    // Don't generate skirts on the corners.
                    continue;
                } else if (isEdge) {
                    heightSample -= skirtHeight;

                    if (isWestEdge) {
                        // The outer loop iterates north to south but the indices are ordered south to north, hence the index flip below
                        index = gridVertexCount + (height - row - 1);
                        longitude -= skirtOffsetPercentage * rectangleWidth;
                    } else if (isSouthEdge) {
                        // Add after west indices. South indices are ordered east to west.
                        index = gridVertexCount + height + (width - col - 1);
                    } else if (isEastEdge) {
                        // Add after west and south indices. East indices are ordered north to south. The index is flipped like above.
                        index = gridVertexCount + height + width + row;
                        longitude += skirtOffsetPercentage * rectangleWidth;
                    } else if (isNorthEdge) {
                        // Add after west, south, and east indices. North indices are ordered west to east.
                        index = gridVertexCount + height + width + height + col;
                    }
                }
            }

            let nX = cosLatitude * longitude.cos();
            let nY = cosLatitude * longitude.sin();

            let kX = radiiSquaredX * nX;
            let kY = radiiSquaredY * nY;

            let gamma = (kX * nX + kY * nY + kZ * nZ).sqrt();
            let oneOverGamma = 1.0 / gamma;

            let rSurfaceX = kX * oneOverGamma;
            let rSurfaceY = kY * oneOverGamma;
            let rSurfaceZ = kZ * oneOverGamma;

            let mut position = DVec3::ZERO;
            position.x = rSurfaceX + nX * heightSample;
            position.y = rSurfaceY + nY * heightSample;
            position.z = rSurfaceZ + nZ * heightSample;
            let cartesian3Scratch = toENU.multiply_by_point(&position);
            minimum = cartesian3Scratch.minimum_by_component(minimum);
            maximum = cartesian3Scratch.maximum_by_component(maximum);

            hMin = hMin.min(heightSample);

            positions[index as usize] = position;
            uvs[index as usize] = DVec2::new(u, v);
            heights[index as usize] = heightSample;

            if (includeWebMercatorT) {
                let mut webMercatorTs = webMercatorTsOption.as_mut().unwrap();
                webMercatorTs[index as usize] = webMercatorT;
            }

            if (includeGeodeticSurfaceNormals) {
                let mut geodeticSurfaceNormals = geodeticSurfaceNormalsOption.as_mut().unwrap();
                geodeticSurfaceNormals[index as usize] =
                    ellipsoid.geodeticSurfaceNormal(&position).unwrap();
            }
        }
    }

    let boundingSphere3D = BoundingSphere::from_points(&positions);
    let mut orientedBoundingBox = OrientedBoundingBox::default();
    if let Some(rectangle) = rectangleOption {
        orientedBoundingBox = OrientedBoundingBox::fromRectangle(
            &rectangle,
            Some(minimumHeight),
            Some(maximumHeight),
            Some(ellipsoid),
        );
    }

    let mut occludeePointInScaledSpace = DVec3::ZERO;
    if (hasRelativeToCenter) {
        let occluder = EllipsoidalOccluder::new(&ellipsoid);
        occludeePointInScaledSpace = occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
            relativeToCenter,
            &positions,
            minimumHeight,
        );
    }

    let aaBox = AxisAlignedBoundingBox::new(minimum, maximum, relativeToCenter);
    let encoding = TerrainEncoding::new(
        relativeToCenter,
        Some(aaBox),
        Some(hMin),
        Some(maximumHeight),
        Some(fromENU),
        false,
        Some(includeWebMercatorT),
        Some(includeGeodeticSurfaceNormals),
        Some(exaggeration),
        Some(exaggerationRelativeHeight),
    );
    let lenth = (vertexCount * encoding.stride as u32) as usize;
    let mut vertices = Vec::with_capacity(lenth); // 预分配内存空间
    vertices.extend(std::iter::repeat(0.).take(lenth));

    let mut bufferIndex: i64 = 0;
    for j in 0..vertexCount {
        let jj = j as usize;
        bufferIndex = encoding.encode(
            &mut vertices,
            bufferIndex,
            &mut positions[jj],
            &uvs[jj],
            heights[jj],
            None,
            {
                if webMercatorTsOption.is_none() {
                    None
                } else {
                    let webMercatorTs = webMercatorTsOption.as_ref().unwrap();
                    Some(webMercatorTs[jj])
                }
            },
            {
                if geodeticSurfaceNormalsOption.is_none() {
                    None
                } else {
                    let geodeticSurfaceNormals = geodeticSurfaceNormalsOption.as_ref().unwrap();
                    Some(&geodeticSurfaceNormals[jj])
                }
            },
        );
    }

    return CreateVerticeReturn {
        vertices: vertices,
        maximumHeight: maximumHeight,
        minimumHeight: minimumHeight,
        encoding: encoding,
        boundingSphere3D: boundingSphere3D,
        orientedBoundingBox: orientedBoundingBox,
        occludeePointInScaledSpace: occludeePointInScaledSpace,
    };
}

#[cfg(test)]
mod tests {
    use crate::{equals_epsilon, lerp, Cartographic, EPSILON7};
    use std::f64::consts::PI;

    use super::*;
    use crate::math::*;
    #[test]
    fn test_little_endian_heights() {
        let width = 3;
        let height = 3;
        let mut structure = HeightmapTerrainStructure::default();
        structure.stride = 3;
        structure.elementsPerHeight = 2;
        structure.elementMultiplier = 10;
        let heightmap: Vec<f64> = [
            1.0, 2.0, 100.0, 3.0, 4.0, 100.0, 5.0, 6.0, 100.0, 7.0, 8.0, 100.0, 9.0, 10.0, 100.0,
            11.0, 12.0, 100.0, 13.0, 14.0, 100.0, 15.0, 16.0, 100.0, 17.0, 18.0, 100.0,
        ]
        .into();
        let nativeRectangle = Rectangle {
            west: 10.0,
            south: 30.0,
            east: 20.0,
            north: 40.0,
        };
        let options = CreateVerticeOptions {
            heightmap: heightmap.clone(),

            width: width,
            height: height,
            skirtHeight: 0.0,
            nativeRectangle: nativeRectangle.clone(),
            rectangle: Some(Rectangle::new(
                10.0.to_radians(),
                30.0.to_radians(),
                20.0.to_radians(),
                40.0.to_radians(),
            )),
            structure: Some(structure.clone()),
            isGeographic: None,
            includeWebMercatorT: None,
            exaggeration: None,
            exaggerationRelativeHeight: None,
            relativeToCenter: None,
            ellipsoid: None,
        };
        let results = create_vertice(options);
        let vertices = results.vertices;

        let ellipsoid = Ellipsoid::WGS84;

        for j in 0..height {
            let mut latitude = lerp(
                nativeRectangle.north,
                nativeRectangle.south,
                compute_u32(j, height - 1),
            );
            latitude = latitude.to_radians();
            for i in 0..width {
                let mut longitude = lerp(
                    nativeRectangle.west,
                    nativeRectangle.east,
                    compute_u32(i, width - 1),
                );
                longitude = longitude.to_radians();

                let heightSampleIndex = ((j * width + i) * structure.stride) as usize;
                let heightSample =
                    heightmap[heightSampleIndex] + heightmap[heightSampleIndex + 1] * 10.0;

                let expectedVertexPosition = ellipsoid.cartographicToCartesian(&Cartographic {
                    longitude: longitude,
                    latitude: latitude,
                    height: heightSample,
                });

                let index = ((j * width + i) * 6) as usize;
                let vertexPosition =
                    DVec3::new(vertices[index], vertices[index + 1], vertices[index + 2]);

                assert!(vertexPosition.equals_epsilon(expectedVertexPosition, Some(1.0), None));
                assert!(vertices[index + 3] == heightSample);
                assert!(equals_epsilon(
                    vertices[(index + 4)],
                    compute_u32(i, width - 1),
                    Some(EPSILON7),
                    None
                ));
                assert!(equals_epsilon(
                    vertices[(index + 5)],
                    1.0 - compute_u32(j, height - 1),
                    Some(EPSILON7),
                    None
                ));
            }
        }
    }
    fn compute(index: i32, num: i32) -> f64 {
        return index as f64 / (num as f64);
    }
    fn compute_u32(index: u32, num: u32) -> f64 {
        return index as f64 / (num as f64);
    }
}
