#![warn(
    clippy::unwrap_used,
    clippy::cast_lossless,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::expect_used
)]

use futures_util::StreamExt;
use std::io;
pub struct CreateMeshJobOutput {
    pub vertices: Vec<f64>,
    pub maximumHeight: f64,
    pub minimumHeight: f64,
    pub encoding: f64,
    pub boundingSphere3D: f64,
    pub orientedBoundingBox: f64,
    pub occludeePointInScaledSpace: f64,
}

pub struct CreateMeshJob {
    pub url: String,
    pub crs: String,
    pub name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

impl bevy_jobs::Job for CreateMeshJob {
    type Outcome = Result<CreateMeshJobOutput, Error>;

    fn name(&self) -> String {
        format!("Fetching '{}'", self.name)
    }

    fn perform(self, ctx: bevy_jobs::Context) -> bevy_jobs::AsyncReturn<Self::Outcome> {
        Box::pin(async move {
            let fetch = async {
                Ok(FetchedFile {
                    bytes: bytes::Bytes::from(bytes),
                    crs: self.crs,
                    name: self.name,
                })
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(fetch)
            }
            #[cfg(target_arch = "wasm32")]
            {
                fetch.await
            }
        })
    }
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}

pub fn create_vertice() {
    let cos = Math.cos;
    let sin = Math.sin;
    let sqrt = Math.sqrt;
    let atan = Math.atan;
    let exp = Math.exp;
    let piOverTwo = CesiumMath.PI_OVER_TWO;
    let toRadians = CesiumMath.toRadians;

    let heightmap = options.heightmap;
    let width = options.width;
    let height = options.height;
    let skirtHeight = options.skirtHeight;
    let hasSkirts = skirtHeight > 0.0;

    let isGeographic = defaultValue(options.isGeographic, true);
    let ellipsoid = defaultValue(options.ellipsoid, Ellipsoid.WGS84);

    let oneOverGlobeSemimajorAxis = 1.0 / ellipsoid.maximumRadius;

    let nativeRectangle = Rectangle.clone(options.nativeRectangle);
    let rectangle = Rectangle.clone(options.rectangle);

    let geographicWest;
    let geographicSouth;
    let geographicEast;
    let geographicNorth;

    if (!defined(rectangle)) {
        if (isGeographic) {
        geographicWest = toRadians(nativeRectangle.west);
        geographicSouth = toRadians(nativeRectangle.south);
        geographicEast = toRadians(nativeRectangle.east);
        geographicNorth = toRadians(nativeRectangle.north);
        } else {
        geographicWest = nativeRectangle.west * oneOverGlobeSemimajorAxis;
        geographicSouth =
            piOverTwo -
            2.0 * atan(exp(-nativeRectangle.south * oneOverGlobeSemimajorAxis));
        geographicEast = nativeRectangle.east * oneOverGlobeSemimajorAxis;
        geographicNorth =
            piOverTwo -
            2.0 * atan(exp(-nativeRectangle.north * oneOverGlobeSemimajorAxis));
        }
    } else {
        geographicWest = rectangle.west;
        geographicSouth = rectangle.south;
        geographicEast = rectangle.east;
        geographicNorth = rectangle.north;
    }

    let relativeToCenter = options.relativeToCenter;
    let hasRelativeToCenter = defined(relativeToCenter);
    relativeToCenter = hasRelativeToCenter ? relativeToCenter : Cartesian3.ZERO;
    let includeWebMercatorT = defaultValue(options.includeWebMercatorT, false);

    let exaggeration = defaultValue(options.exaggeration, 1.0);
    let exaggerationRelativeHeight = defaultValue(
        options.exaggerationRelativeHeight,
        0.0
    );
    let hasExaggeration = exaggeration !== 1.0;
    let includeGeodeticSurfaceNormals = hasExaggeration;

    let structure = defaultValue(
        options.structure,
        HeightmapTessellator.
        
    );
    let heightScale = defaultValue(
        structure.heightScale,
        HeightmapTessellator.DEFAULT_STRUCTURE.heightScale
    );
    let heightOffset = defaultValue(
        structure.heightOffset,
        HeightmapTessellator.DEFAULT_STRUCTURE.heightOffset
    );
    let elementsPerHeight = defaultValue(
        structure.elementsPerHeight,
        HeightmapTessellator.DEFAULT_STRUCTURE.elementsPerHeight
    );
    let stride = defaultValue(
        structure.stride,
        HeightmapTessellator.DEFAULT_STRUCTURE.stride
    );
    let elementMultiplier = defaultValue(
        structure.elementMultiplier,
        HeightmapTessellator.DEFAULT_STRUCTURE.elementMultiplier
    );
    let isBigEndian = defaultValue(
        structure.isBigEndian,
        HeightmapTessellator.DEFAULT_STRUCTURE.isBigEndian
    );

    let rectangleWidth = Rectangle.computeWidth(nativeRectangle);
    let rectangleHeight = Rectangle.computeHeight(nativeRectangle);

    let granularityX = rectangleWidth / (width - 1);
    let granularityY = rectangleHeight / (height - 1);

    if (!isGeographic) {
        rectangleWidth *= oneOverGlobeSemimajorAxis;
        rectangleHeight *= oneOverGlobeSemimajorAxis;
    }

    let radiiSquared = ellipsoid.radiiSquared;
    let radiiSquaredX = radiiSquared.x;
    let radiiSquaredY = radiiSquared.y;
    let radiiSquaredZ = radiiSquared.z;

    let minimumHeight = 65536.0;
    let maximumHeight = -65536.0;

    let fromENU = Transforms.eastNorthUpToFixedFrame(
        relativeToCenter,
        ellipsoid
    );
    let toENU = Matrix4.inverseTransformation(fromENU, matrix4Scratch);

    let southMercatorY;
    let oneOverMercatorHeight;
    if (includeWebMercatorT) {
        southMercatorY = WebMercatorProjection.geodeticLatitudeToMercatorAngle(
        geographicSouth
        );
        oneOverMercatorHeight =
        1.0 /
        (WebMercatorProjection.geodeticLatitudeToMercatorAngle(geographicNorth) -
            southMercatorY);
    }

    let mut minimum = minimumScratch;
    minimum.x = Number.POSITIVE_INFINITY;
    minimum.y = Number.POSITIVE_INFINITY;
    minimum.z = Number.POSITIVE_INFINITY;

    let mut maximum = maximumScratch;
    maximum.x = Number.NEGATIVE_INFINITY;
    maximum.y = Number.NEGATIVE_INFINITY;
    maximum.z = Number.NEGATIVE_INFINITY;

    let hMin = Number.POSITIVE_INFINITY;

    let gridVertexCount = width * height;
    let edgeVertexCount = skirtHeight > 0.0 ? width * 2 + height * 2 : 0;
    let vertexCount = gridVertexCount + edgeVertexCount;

    let positions = new Array(vertexCount);
    let heights = new Array(vertexCount);
    let uvs = new Array(vertexCount);
    let webMercatorTs = includeWebMercatorT ? new Array(vertexCount) : [];
    let geodeticSurfaceNormals = includeGeodeticSurfaceNormals
        ? new Array(vertexCount)
        : [];

    let mut startRow = 0;
    let mut endRow = height;
    let mut startCol = 0;
    let mut endCol = width;

    if (hasSkirts) {
        startRow-=1;
        endRow+=1;
        startCol-=1;
        endCol+=1;
    }

    let skirtOffsetPercentage = 0.00001;

    for (let rowIndex = startRow; rowIndex < endRow; ++rowIndex) {
        let row = rowIndex;
        if (row < 0) {
        row = 0;
        }
        if (row >= height) {
        row = height - 1;
        }

        let latitude = nativeRectangle.north - granularityY * row;

        if (!isGeographic) {
        latitude =
            piOverTwo - 2.0 * atan(exp(-latitude * oneOverGlobeSemimajorAxis));
        } else {
        latitude = toRadians(latitude);
        }

        let v = (latitude - geographicSouth) / (geographicNorth - geographicSouth);
        v = CesiumMath.clamp(v, 0.0, 1.0);

        let isNorthEdge = rowIndex === startRow;
        let isSouthEdge = rowIndex === endRow - 1;
        if (skirtHeight > 0.0) {
        if (isNorthEdge) {
            latitude += skirtOffsetPercentage * rectangleHeight;
        } else if (isSouthEdge) {
            latitude -= skirtOffsetPercentage * rectangleHeight;
        }
        }

        let cosLatitude = cos(latitude);
        let nZ = sin(latitude);
        let kZ = radiiSquaredZ * nZ;

        let webMercatorT;
        if (includeWebMercatorT) {
        webMercatorT =
            (WebMercatorProjection.geodeticLatitudeToMercatorAngle(latitude) -
            southMercatorY) *
            oneOverMercatorHeight;
        }

        for (let colIndex = startCol; colIndex < endCol; ++colIndex) {
        let col = colIndex;
        if (col < 0) {
            col = 0;
        }
        if (col >= width) {
            col = width - 1;
        }

        let terrainOffset = row * (width * stride) + col * stride;

        let heightSample;
        if (elementsPerHeight === 1) {
            heightSample = heightmap[terrainOffset];
        } else {
            heightSample = 0;

            let elementOffset;
            if (isBigEndian) {
            for (
                elementOffset = 0;
                elementOffset < elementsPerHeight;
                ++elementOffset
            ) {
                heightSample =
                heightSample * elementMultiplier +
                heightmap[terrainOffset + elementOffset];
            }
            } else {
            for (
                elementOffset = elementsPerHeight - 1;
                elementOffset >= 0;
                --elementOffset
            ) {
                heightSample =
                heightSample * elementMultiplier +
                heightmap[terrainOffset + elementOffset];
            }
            }
        }

        heightSample = heightSample * heightScale + heightOffset;

        maximumHeight = Math.max(maximumHeight, heightSample);
        minimumHeight = Math.min(minimumHeight, heightSample);

        let longitude = nativeRectangle.west + granularityX * col;

        if (!isGeographic) {
            longitude = longitude * oneOverGlobeSemimajorAxis;
        } else {
            longitude = toRadians(longitude);
        }

        let u = (longitude - geographicWest) / (geographicEast - geographicWest);
        u = CesiumMath.clamp(u, 0.0, 1.0);

        let index = row * width + col;

        if (skirtHeight > 0.0) {
            let isWestEdge = colIndex === startCol;
            let isEastEdge = colIndex === endCol - 1;
            let isEdge = isNorthEdge || isSouthEdge || isWestEdge || isEastEdge;
            let isCorner =
            (isNorthEdge || isSouthEdge) && (isWestEdge || isEastEdge);
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

        let nX = cosLatitude * cos(longitude);
        let nY = cosLatitude * sin(longitude);

        let kX = radiiSquaredX * nX;
        let kY = radiiSquaredY * nY;

        let gamma = sqrt(kX * nX + kY * nY + kZ * nZ);
        let oneOverGamma = 1.0 / gamma;

        let rSurfaceX = kX * oneOverGamma;
        let rSurfaceY = kY * oneOverGamma;
        let rSurfaceZ = kZ * oneOverGamma;

        let position = new Cartesian3();
        position.x = rSurfaceX + nX * heightSample;
        position.y = rSurfaceY + nY * heightSample;
        position.z = rSurfaceZ + nZ * heightSample;

        Matrix4.multiplyByPoint(toENU, position, cartesian3Scratch);
        Cartesian3.minimumByComponent(cartesian3Scratch, minimum, minimum);
        Cartesian3.maximumByComponent(cartesian3Scratch, maximum, maximum);
        hMin = Math.min(hMin, heightSample);

        positions[index] = position;
        uvs[index] = new Cartesian2(u, v);
        heights[index] = heightSample;

        if (includeWebMercatorT) {
            webMercatorTs[index] = webMercatorT;
        }

        if (includeGeodeticSurfaceNormals) {
            geodeticSurfaceNormals[index] = ellipsoid.geodeticSurfaceNormal(
            position
            );
        }
        }
    }

    let boundingSphere3D = BoundingSphere.fromPoints(positions);
    let orientedBoundingBox;
    if (defined(rectangle)) {
        orientedBoundingBox = OrientedBoundingBox.fromRectangle(
        rectangle,
        minimumHeight,
        maximumHeight,
        ellipsoid
        );
    }

    let occludeePointInScaledSpace;
    if (hasRelativeToCenter) {
        let occluder = new EllipsoidalOccluder(ellipsoid);
        occludeePointInScaledSpace = occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
        relativeToCenter,
        positions,
        minimumHeight
        );
    }

    let aaBox = new AxisAlignedBoundingBox(minimum, maximum, relativeToCenter);
    let encoding = new TerrainEncoding(
        relativeToCenter,
        aaBox,
        hMin,
        maximumHeight,
        fromENU,
        false,
        includeWebMercatorT,
        includeGeodeticSurfaceNormals,
        exaggeration,
        exaggerationRelativeHeight
    );
    let vertices = new Float32Array(vertexCount * encoding.stride);

    let bufferIndex = 0;
    for (let j = 0; j < vertexCount; ++j) {
        bufferIndex = encoding.encode(
        vertices,
        bufferIndex,
        positions[j],
        uvs[j],
        heights[j],
        undefined,
        webMercatorTs[j],
        geodeticSurfaceNormals[j]
        );
    }

    return {
        vertices: vertices,
        maximumHeight: maximumHeight,
        minimumHeight: minimumHeight,
        encoding: encoding,
        boundingSphere3D: boundingSphere3D,
        orientedBoundingBox: orientedBoundingBox,
        occludeePointInScaledSpace: occludeePointInScaledSpace,
    };
}
