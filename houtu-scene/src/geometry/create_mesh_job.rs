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

pub fn create_vertice(options: CreateVerticeOptions) {
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

    let rectangleWidth = nativeRectangle.compute_width();
    let rectangleHeight = nativeRectangle.compute_height();

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

    let minimum_height: f64 = 65536.0;
    let maximum_height: f64 = -65536.0;

    let from_enu = eastNorthUpToFixedFrame(relativeToCenter, Some(ellipsoid));
    let to_enu = from_enu.inverse_transformation();
    let webMercatorProjection = WebMercatorProjection::default();
    let mut south_mercator_y;
    let mut one_over_mercator_height;
    if includeWebMercatorT {
        south_mercator_y =
            webMercatorProjection.geodetic_latitude_to_mercator_angle(geographicSouth);
        one_over_mercator_height = 1.0
            / (webMercatorProjection.geodetic_latitude_to_mercator_angle(geographicNorth)
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

    let grid_vertex_count: i64 = width * height;
    let edge_vertex_count: i64 = {
        if skirt_height > 0.0 {
            width * 2 + height * 2
        } else {
            0
        }
    };

    let vertex_count = grid_vertex_count + edge_vertex_count;
    let mut positions = Vec::with_capacity(vertex_count); // 预分配内存空间
    positions.extend(std::iter::repeat(0.).take(vertex_count)); // 填充初始值
    let mut heights = Vec::with_capacity(vertex_count); // 预分配内存空间
    heights.extend(std::iter::repeat(0.).take(vertex_count)); // 填充初始值
    let mut uvs = Vec::with_capacity(vertex_count); // 预分配内存空间
    uvs.extend(std::iter::repeat(0.)..take(vertex_count)); // 填充初始值

    let webMercatorTs = {
        if includeWebMercatorT {
            let mut tmp = Vec::with_capacity(vertex_count); // 预分配内存空间
            tmp.extend(std::iter::repeat(0.)..take(vertex_count)); // 填充初始值
            tmp
        } else {
            vec![]
        }
    };
    let geodeticSurfaceNormals = {
        if includeGeodeticSurfaceNormals {
            let mut tmp = Vec::with_capacity(vertex_count); // 预分配内存空间
            tmp.extend(std::iter::repeat(0.)..take(vertex_count)); // 填充初始值
            tmp
        } else {
            vec![]
        }
    };
    let mut startRow = 0;
    let mut endRow = height;
    let mut startCol = 0;
    let mut endCol = width;

    if hasSkirts {
        startRow -= 1;
        endRow += 1;
        startCol -= 1;
        endCol += 1;
    }

    let skirtOffsetPercentage = 0.00001;
    for rowIndex in startRow..endRow {
        let row = rowIndex;
        if row < 0 {
            row = 0;
        }
        if row >= height {
            row = height - 1;
        }

        let mut latitude = nativeRectangle.north - granularityY * row as f64;

        if !isGeographic {
            latitude = piOverTwo - 2.0 * ((-latitude * oneOverGlobeSemimajorAxis).exp()).atan();
        } else {
            latitude = (latitude).to_radians();
        }

        let v = (latitude - geographicSouth) / (geographicNorth - geographicSouth);
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

        let cos_latitude = (latitude).cos();
        let nZ = (latitude).sin();
        let kZ = radiiSquaredZ * nZ;

        let web_mercator_t;
        if includeWebMercatorT {
            web_mercator_t = (webMercatorProjection.geodetic_latitude_to_mercator_angle(latitude)
                - south_mercator_y)
                * one_over_mercator_height;
        }
        for colIndex in startCol..endCol {
            let col = colIndex;
            if col < 0 {
                col = 0;
            }
            if col >= width {
                col = width - 1;
            }

            let terrainOffset = row * (width * stride) + col * stride;

            let heightSample: f64;
            if elements_per_height == 1 {
                heightSample = heightmap[terrainOffset as usize];
            } else {
                heightSample = 0.;

                let elementOffset;
                if is_big_endian {
                    for elementOffset in 0..elements_per_height {
                        heightSample = heightSample * element_multiplier
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                } else {
                    //可能会出问题，注意
                    for elementOffset in (0..elements_per_height).rev() {
                        heightSample = heightSample * element_multiplier
                            + heightmap[(terrainOffset + elementOffset) as usize];
                    }
                }
            }

            heightSample = heightSample * height_scale + height_offset;

            maximum_height = maximum_height.max(heightSample);
            minimum_height = minimum_height.min(heightSample);

            let longitude = nativeRectangle.west + granularityX * (col as f64);

            if !isGeographic {
                longitude = longitude * oneOverGlobeSemimajorAxis;
            } else {
                longitude = (longitude).to_radians();
            }

            let u = (longitude - geographicWest) / (geographicEast - geographicWest);
            u = u.clamp(0.0, 1.0);

            let index = row * width + col;

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
                        index = grid_vertex_count + (height - row - 1);
                        longitude -= skirtOffsetPercentage * rectangleWidth;
                    } else if isSouthEdge {
                        // Add after west indices. South indices are ordered east to west.
                        index = grid_vertex_count + height + (width - col - 1);
                    } else if isEastEdge {
                        // Add after west and south indices. East indices are ordered north to south. The index is flipped like above.
                        index = grid_vertex_count + height + width + row;
                        longitude += skirtOffsetPercentage * rectangleWidth;
                    } else if isNorthEdge {
                        // Add after west, south, and east indices. North indices are ordered west to east.
                        index = grid_vertex_count + height + width + height + col;
                    }
                }
            }

            let nX = cos_latitude * longitude.cos();
            let nY = cos_latitude * longitude.sin();

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
            let cartesian3_scratch = to_enu.multiply_by_point(position);
            minimum = cartesian3_scratch.minimum_by_component(minimum);
            maximum = cartesian3_scratch.maximum_by_component(maximum);

            hMin = hMin.min(heightSample);

            positions[index as usize] = position;
            uvs[index as usize] = DVec2::new(u, v);
            heights[index as usize] = heightSample;

            if includeWebMercatorT {
                webMercatorTs[index as usize] = web_mercator_t;
            }

            if includeGeodeticSurfaceNormals {
                geodeticSurfaceNormals[index as usize] =
                    ellipsoid.geodetic_surface_normal(&position);
            }
        }
    }

    let bounding_sphere_3d = BoundingSphere.from_points(positions);
    let oriented_bounding_box;
    if defined(rectangle) {
        oriented_bounding_box = OrientedBoundingBox::from_rectangle(
            rectangle,
            minimum_height,
            maximum_height,
            ellipsoid,
        );
    }

    let occludee_point_in_scaled_space;
    if hasRelativeToCenter {
        let occluder = EllipsoidalOccluder::new(&ellipsoid);
        occludee_point_in_scaled_space = occluder
            .compute_horizon_culling_point_possibly_under_ellipsoid(
                relativeToCenter,
                positions,
                minimum_height,
            );
    }

    let aaBox = AxisAlignedBoundingBox::new(minimum, maximum, relativeToCenter);
    // let encoding = new TerrainEncoding(
    //   relativeToCenter,
    //   aaBox,
    //   hMin,
    //   maximum_height,
    //   from_enu,
    //   false,
    //   includeWebMercatorT,
    //   includeGeodeticSurfaceNormals,
    //   exaggeration,
    //   exaggeration_relative_height
    // );
    // let vertices = new Float32Array(vertex_count * encoding.stride);

    // let buffer_index = 0;
    // for (let j = 0; j < vertex_count; j+=1) {
    //   buffer_index = encoding.encode(
    //     vertices,
    //     buffer_index,
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
    //   maximum_height: maximum_height,
    //   minimum_height: minimum_height,
    //   encoding: encoding,
    //   bounding_sphere_3d: bounding_sphere_3d,
    //   oriented_bounding_box: oriented_bounding_box,
    //   occludee_point_in_scaled_space: occludee_point_in_scaled_space,
    // };
}
