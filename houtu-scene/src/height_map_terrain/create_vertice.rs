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
    pub maximum_height: f64,
    pub minimum_height: f64,
    pub encoding: f64,
    pub bounding_sphere_3d: f64,
    pub oriented_bounding_box: f64,
    pub occludee_point_in_scaled_space: f64,
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
pub struct CreateVerticeOptions<'a> {
    pub heightmap: &'a mut Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub skirt_height: f64,
    pub nativeRectangle: Rectangle,
    pub exaggeration: Option<f64>,
    pub exaggeration_relative_height: Option<f64>,
    pub rectangle: Option<Rectangle>,
    pub isGeographic: Option<bool>,
    pub relativeToCenter: Option<DVec3>,
    pub ellipsoid: Option<Ellipsoid>,
    pub structure: Option<HeightmapTerrainStructure>,
    pub includeWebMercatorT: Option<bool>,
}
pub struct CreateVerticeReturn {
    pub vertices: Vec<f32>,
    pub maximum_height: f64,
    pub minimum_height: f64,
    pub encoding: TerrainEncoding,
    pub bounding_sphere_3d: BoundingSphere,
    pub oriented_bounding_box: OrientedBoundingBox,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub relativeToCenter: Option<DVec3>,
}
pub fn create_vertice(options: CreateVerticeOptions) -> CreateVerticeReturn {
    let piOverTwo = FRAC_PI_2;
    let heightmap = options.heightmap;
    let width = options.width;
    let height = options.height;
    let skirt_height = options.skirt_height;
    let hasSkirts = skirt_height > 0.0;
    let isGeographic = options.isGeographic.unwrap_or(true);
    let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

    let oneOverGlobeSemimajorAxis = 1.0 / ellipsoid.maximum_radius;

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
    let exaggeration_relative_height = options.exaggeration_relative_height.unwrap_or(0.0);
    let hasExaggeration = exaggeration != 1.0;
    let includeGeodeticSurfaceNormals = hasExaggeration;
    let structore = options
        .structure
        .unwrap_or(HeightmapTerrainStructure::default());

    let height_scale = structore.height_scale;
    let height_offset = structore.height_offset;
    let elements_per_height = structore.elements_per_height;
    let stride = structore.stride;
    let element_multiplier = structore.element_multiplier;
    let is_big_endian = structore.is_big_endian;

    let mut rectangleWidth = nativeRectangle.compute_width();
    let mut rectangleHeight = nativeRectangle.compute_height();

    let granularityX = rectangleWidth / (width as f64 - 1.);
    let granularityY = rectangleHeight / (height as f64 - 1.);

    if !isGeographic {
        rectangleWidth *= oneOverGlobeSemimajorAxis;
        rectangleHeight *= oneOverGlobeSemimajorAxis;
    }

    let radii_squared = ellipsoid.radii_squared;
    let radiiSquaredX = radii_squared.x;
    let radiiSquaredY = radii_squared.y;
    let radiiSquaredZ = radii_squared.z;

    let mut minimum_height: f64 = 65536.0;
    let mut maximum_height: f64 = -65536.0;

    let fromENU = eastNorthUpToFixedFrame(&relativeToCenter, Some(ellipsoid));
    let toENU = fromENU.inverse_transformation();
    let webMercatorProjection = WebMercatorProjection::default();
    let mut south_mercator_y = 0.;
    let mut one_over_mercator_height = 0.;
    if includeWebMercatorT {
        south_mercator_y =
            webMercatorProjection.geodeticLatitude_to_mercator_angle(geographicSouth);
        one_over_mercator_height = 1.0
            / (webMercatorProjection.geodeticLatitude_to_mercator_angle(geographicNorth)
                - south_mercator_y);
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
        if skirt_height > 0.0 {
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
    let mut startRow: i32 = 0;
    let mut endRow: i32 = height as i32;
    let mut startCol: i32 = 0;
    let mut endCol: i32 = width as i32;

    if hasSkirts {
        startRow -= 1;
        endRow += 1;
        startCol -= 1;
        endCol += 1;
    }

    let skirtOffsetPercentage = 0.00001;
    for rowIndex in startRow..endRow {
        let mut row = rowIndex;
        if row < 0 {
            row = 0;
        }
        if row >= height as i32 {
            row = (height - 1) as i32;
        }

        let mut latitude = nativeRectangle.north - granularityY * row as f64;

        if !isGeographic {
            latitude = piOverTwo - 2.0 * ((-latitude * oneOverGlobeSemimajorAxis).exp()).atan();
        } else {
            latitude = (latitude).to_radians();
        }

        let mut v = (latitude - geographicSouth) / (geographicNorth - geographicSouth);
        v = v.clamp(0.0, 1.0);

        let isNorthEdge = rowIndex == startRow;
        let isSouthEdge = rowIndex == endRow - 1;
        if skirt_height > 0.0 {
            if isNorthEdge {
                latitude += skirtOffsetPercentage * rectangleHeight;
            } else if isSouthEdge {
                latitude -= skirtOffsetPercentage * rectangleHeight;
            }
        }

        let cosLatitude = (latitude).cos();
        let nZ = (latitude).sin();
        let kZ = radiiSquaredZ * nZ;

        let mut web_mercator_t: f64 = 0.;
        if includeWebMercatorT {
            web_mercator_t = (webMercatorProjection.geodeticLatitude_to_mercator_angle(latitude)
                - south_mercator_y)
                * one_over_mercator_height;
        }
        for colIndex in startCol..endCol {
            let mut col = colIndex;
            if col < 0 {
                col = 0;
            }
            if col >= width as i32 {
                col = (width - 1) as i32;
            }

            let terrainOffset = (row as u32) * (width * stride) + (col as u32) * stride;

            let mut heightSample: f64;
            if elements_per_height == 1 {
                heightSample = heightmap[terrainOffset as usize] as f64;
            } else {
                heightSample = 0.;

                if is_big_endian {
                    for elementOffset in 0..elements_per_height {
                        heightSample = heightSample * element_multiplier as f64
                            + heightmap[(terrainOffset + elementOffset) as usize] as f64;
                    }
                } else {
                    //可能会出问题，注意
                    for elementOffset in (0..elements_per_height).rev() {
                        heightSample = heightSample * element_multiplier as f64
                            + heightmap[(terrainOffset + elementOffset) as usize] as f64;
                    }
                }
            }

            heightSample = heightSample * height_scale + height_offset;

            maximum_height = maximum_height.max(heightSample);
            minimum_height = minimum_height.min(heightSample);

            let mut longitude = nativeRectangle.west + granularityX * (col as f64);

            if !isGeographic {
                longitude = longitude * oneOverGlobeSemimajorAxis;
            } else {
                longitude = (longitude).to_radians();
            }

            let mut u = (longitude - geographicWest) / (geographicEast - geographicWest);
            u = u.clamp(0.0, 1.0);

            let mut index = (row as u32) * width + (col as u32);

            if skirt_height > 0.0 {
                let isWestEdge = colIndex == startCol;
                let isEastEdge = colIndex == endCol - 1;
                let isEdge = isNorthEdge || isSouthEdge || isWestEdge || isEastEdge;
                let isCorner = (isNorthEdge || isSouthEdge) && (isWestEdge || isEastEdge);
                if isCorner {
                    // Don't generate skirts on the corners.
                    continue;
                } else if isEdge {
                    heightSample -= skirt_height;

                    if isWestEdge {
                        // The outer loop iterates north to south but the indices are ordered south to north, hence the index flip below
                        index = gridVertexCount + (height - (row as u32) - 1);
                        longitude -= skirtOffsetPercentage * rectangleWidth;
                    } else if isSouthEdge {
                        // Add after west indices. South indices are ordered east to west.
                        index = gridVertexCount + height + (width - (col as u32) - 1);
                    } else if isEastEdge {
                        // Add after west and south indices. East indices are ordered north to south. The index is flipped like above.
                        index = gridVertexCount + height + width + (row as u32);
                        longitude += skirtOffsetPercentage * rectangleWidth;
                    } else if isNorthEdge {
                        // Add after west, south, and east indices. North indices are ordered west to east.
                        index = gridVertexCount + height + width + height + (col as u32);
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

            if includeWebMercatorT {
                let mut webMercatorTs = webMercatorTsOption.as_mut().unwrap();
                webMercatorTs[index as usize] = web_mercator_t;
            }

            if includeGeodeticSurfaceNormals {
                let mut geodeticSurfaceNormals = geodeticSurfaceNormalsOption.as_mut().unwrap();
                geodeticSurfaceNormals[index as usize] =
                    ellipsoid.geodeticSurfaceNormal(&position).unwrap();
            }
        }
    }

    let bounding_sphere_3d = BoundingSphere::from_points(&positions);
    let mut oriented_bounding_box = OrientedBoundingBox::default();
    if let Some(rectangle) = rectangleOption {
        oriented_bounding_box = OrientedBoundingBox::fromRectangle(
            &rectangle,
            Some(minimum_height),
            Some(maximum_height),
            Some(&ellipsoid),
        );
    }

    let mut occludee_point_in_scaled_space: Option<DVec3> = None;
    if hasRelativeToCenter {
        let occluder = EllipsoidalOccluder::new(&ellipsoid);
        occludee_point_in_scaled_space = occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
            &relativeToCenter,
            &positions,
            minimum_height,
        );
    }

    let aaBox = AxisAlignedBoundingBox::new(minimum, maximum, relativeToCenter);
    let encoding = TerrainEncoding::new(
        relativeToCenter,
        Some(aaBox),
        Some(hMin),
        Some(maximum_height),
        Some(fromENU),
        false,
        Some(includeWebMercatorT),
        Some(includeGeodeticSurfaceNormals),
        Some(exaggeration),
        Some(exaggeration_relative_height),
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
        relativeToCenter: options.relativeToCenter,
        vertices: vertices,
        maximum_height: maximum_height,
        minimum_height: minimum_height,
        encoding: encoding,
        bounding_sphere_3d: bounding_sphere_3d,
        oriented_bounding_box: oriented_bounding_box,
        occludee_point_in_scaled_space: occludee_point_in_scaled_space,
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
        structure.elements_per_height = 2;
        structure.element_multiplier = 10;
        let mut heightmap: Vec<f32> = [
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
            heightmap: &mut heightmap,

            width: width,
            height: height,
            skirt_height: 0.0,
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
            exaggeration_relative_height: None,
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
                    (heightmap[heightSampleIndex] + heightmap[heightSampleIndex + 1] * 10.0) as f64;

                let expectedVertexPosition = ellipsoid.cartographicToCartesian(&Cartographic {
                    longitude: longitude,
                    latitude: latitude,
                    height: heightSample,
                });

                let index = ((j * width + i) * 6) as usize;
                let vertexPosition = DVec3::new(
                    vertices[index] as f64,
                    vertices[index + 1] as f64,
                    vertices[index + 2] as f64,
                );

                assert!(vertexPosition.equals_epsilon(expectedVertexPosition, Some(1.0), None));
                assert!(vertices[index + 3] == heightSample as f32);
                assert!(equals_epsilon(
                    vertices[(index + 4)] as f64,
                    compute_u32(i, width - 1),
                    Some(EPSILON7),
                    None
                ));
                assert!(equals_epsilon(
                    vertices[(index + 5)] as f64,
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
