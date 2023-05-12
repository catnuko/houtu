#![warn(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::expect_used
)]
use crate::{
    ellipsoidal_occluder::EllipsoidalOccluder,
    geometry::{AxisAlignedBoundingBox, OrientedBoundingBox},
    math::{eastNorthUpToFixedFrame, Cartesian3, Matrix4},
    web_mercator_projection::WebMercatorProjection,
};

use super::*;
use bevy::{
    math::{DVec2, DVec3},
    render::primitives::Aabb,
};
use std::{
    f64::{MAX, MIN_POSITIVE},
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

pub fn create_vertice(options: CreateVerticeOptions) {
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

    let rectangleWidth = nativeRectangle.computeWidth();
    let rectangleHeight = nativeRectangle.computeHeight();

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

    let minimumHeight: f64 = 65536.0;
    let maximumHeight: f64 = -65536.0;

    let fromENU = eastNorthUpToFixedFrame(relativeToCenter, Some(ellipsoid));
    let toENU = fromENU.inverse_transformation();
    let webMercatorProjection = WebMercatorProjection::default();
    let mut southMercatorY;
    let mut oneOverMercatorHeight;
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

    let gridVertexCount: i64 = width * height;
    let edgeVertexCount: i64 = {
        if skirtHeight > 0.0 {
            width * 2 + height * 2
        } else {
            0
        }
    };

    let vertexCount = gridVertexCount + edgeVertexCount;
    let mut positions = Vec::with_capacity(vertexCount); // 预分配内存空间
    positions.extend(std::iter::repeat(0.).take(vertexCount)); // 填充初始值
    let mut heights = Vec::with_capacity(vertexCount); // 预分配内存空间
    heights.extend(std::iter::repeat(0.).take(vertexCount)); // 填充初始值
    let mut uvs = Vec::with_capacity(vertexCount); // 预分配内存空间
    uvs.extend(std::iter::repeat(0.)..take(vertexCount)); // 填充初始值

    let webMercatorTs = {
        if includeWebMercatorT {
            let mut tmp = Vec::with_capacity(vertexCount); // 预分配内存空间
            tmp.extend(std::iter::repeat(0.)..take(vertexCount)); // 填充初始值
            tmp
        } else {
            vec![]
        }
    };
    let geodeticSurfaceNormals = {
        if includeGeodeticSurfaceNormals {
            let mut tmp = Vec::with_capacity(vertexCount); // 预分配内存空间
            tmp.extend(std::iter::repeat(0.)..take(vertexCount)); // 填充初始值
            tmp
        } else {
            vec![]
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
        let row = rowIndex;
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

        let v = (latitude - geographicSouth) / (geographicNorth - geographicSouth);
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

        let webMercatorT;
        if (includeWebMercatorT) {
            webMercatorT = (webMercatorProjection.geodeticLatitude_to_mercator_angle(latitude)
                - southMercatorY)
                * oneOverMercatorHeight;
        }
        for colIndex in startCol..endCol {
            let col = colIndex;
            if (col < 0) {
                col = 0;
            }
            if (col >= width) {
                col = width - 1;
            }

            let terrainOffset = row * (width * stride) + col * stride;

            let heightSample: f64;
            if (elementsPerHeight == 1) {
                heightSample = heightmap[terrainOffset as usize];
            } else {
                heightSample = 0.;

                let elementOffset;
                if (isBigEndian) {
                    for elementOffset in 0..elementsPerHeight {
                        heightSample = heightSample * elementMultiplier
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                } else {
                    //可能会出问题，注意
                    for elementOffset in (0..elementsPerHeight).rev() {
                        heightSample = heightSample * elementMultiplier
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                }
            }

            heightSample = heightSample * heightScale + heightOffset;

            maximumHeight = maximumHeight.max(heightSample);
            minimumHeight = minimumHeight.min(heightSample);

            let longitude = nativeRectangle.west + granularityX * (col as f64);

            if (!isGeographic) {
                longitude = longitude * oneOverGlobeSemimajorAxis;
            } else {
                longitude = (longitude).to_radians();
            }

            let u = (longitude - geographicWest) / (geographicEast - geographicWest);
            u = u.clamp(0.0, 1.0);

            let index = row * width + col;

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

            let position = DVec3::ZERO;
            position.x = rSurfaceX + nX * heightSample;
            position.y = rSurfaceY + nY * heightSample;
            position.z = rSurfaceZ + nZ * heightSample;
            let cartesian3Scratch = toENU.multiply_by_point(position);
            minimum = cartesian3Scratch.minimum_by_component(minimum);
            maximum = cartesian3Scratch.maximum_by_component(maximum);

            hMin = hMin.min(heightSample);

            positions[index as usize] = position;
            uvs[index as usize] = DVec2::new(u, v);
            heights[index as usize] = heightSample;

            if (includeWebMercatorT) {
                webMercatorTs[index as usize] = webMercatorT;
            }

            if (includeGeodeticSurfaceNormals) {
                geodeticSurfaceNormals[index as usize] = ellipsoid.geodeticSurfaceNormal(&position);
            }
        }
    }

    let boundingSphere3D = BoundingSphere.fromPoints(positions);
    let orientedBoundingBox;
    if (defined(rectangle)) {
        orientedBoundingBox =
            OrientedBoundingBox::fromRectangle(rectangle, minimumHeight, maximumHeight, ellipsoid);
    }

    let occludeePointInScaledSpace;
    if (hasRelativeToCenter) {
        let occluder = EllipsoidalOccluder::new(&ellipsoid);
        occludeePointInScaledSpace = occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
            relativeToCenter,
            positions,
            minimumHeight,
        );
    }

    let aaBox = AxisAlignedBoundingBox::new(minimum, maximum, relativeToCenter);
    // let encoding = new TerrainEncoding(
    //   relativeToCenter,
    //   aaBox,
    //   hMin,
    //   maximumHeight,
    //   fromENU,
    //   false,
    //   includeWebMercatorT,
    //   includeGeodeticSurfaceNormals,
    //   exaggeration,
    //   exaggerationRelativeHeight
    // );
    // let vertices = new Float32Array(vertexCount * encoding.stride);

    // let bufferIndex = 0;
    // for (let j = 0; j < vertexCount; j+=1) {
    //   bufferIndex = encoding.encode(
    //     vertices,
    //     bufferIndex,
    //     positions[j],
    //     uvs[j],
    //     heights[j],
    //     undefined,
    //     webMercatorTs[j],
    //     geodeticSurfaceNormals[j]
    //   );
    // }

    // return {
    //   vertices: vertices,
    //   maximumHeight: maximumHeight,
    //   minimumHeight: minimumHeight,
    //   encoding: encoding,
    //   boundingSphere3D: boundingSphere3D,
    //   orientedBoundingBox: orientedBoundingBox,
    //   occludeePointInScaledSpace: occludeePointInScaledSpace,
    // };
}
