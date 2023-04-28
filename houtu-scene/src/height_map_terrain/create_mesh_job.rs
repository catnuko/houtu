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
    
    
    const cos = Math.cos;
    const sin = Math.sin;
    const sqrt = Math.sqrt;
    const atan = Math.atan;
    const exp = Math.exp;
    const piOverTwo = CesiumMath.PI_OVER_TWO;
    const toRadians = CesiumMath.toRadians;

    const heightmap = options.heightmap;
    const width = options.width;
    const height = options.height;
    const skirtHeight = options.skirtHeight;
    const hasSkirts = skirtHeight > 0.0;

    const isGeographic = defaultValue(options.isGeographic, true);
    const ellipsoid = defaultValue(options.ellipsoid, Ellipsoid.WGS84);

    const oneOverGlobeSemimajorAxis = 1.0 / ellipsoid.maximumRadius;

    const nativeRectangle = Rectangle.clone(options.nativeRectangle);
    const rectangle = Rectangle.clone(options.rectangle);

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
    const hasRelativeToCenter = defined(relativeToCenter);
    relativeToCenter = hasRelativeToCenter ? relativeToCenter : Cartesian3.ZERO;
    const includeWebMercatorT = defaultValue(options.includeWebMercatorT, false);

    const exaggeration = defaultValue(options.exaggeration, 1.0);
    const exaggerationRelativeHeight = defaultValue(
        options.exaggerationRelativeHeight,
        0.0
    );
    const hasExaggeration = exaggeration !== 1.0;
    const includeGeodeticSurfaceNormals = hasExaggeration;

    const structure = defaultValue(
        options.structure,
        HeightmapTessellator.DEFAULT_STRUCTURE
    );
    const heightScale = defaultValue(
        structure.heightScale,
        HeightmapTessellator.DEFAULT_STRUCTURE.heightScale
    );
    const heightOffset = defaultValue(
        structure.heightOffset,
        HeightmapTessellator.DEFAULT_STRUCTURE.heightOffset
    );
    const elementsPerHeight = defaultValue(
        structure.elementsPerHeight,
        HeightmapTessellator.DEFAULT_STRUCTURE.elementsPerHeight
    );
    const stride = defaultValue(
        structure.stride,
        HeightmapTessellator.DEFAULT_STRUCTURE.stride
    );
    const elementMultiplier = defaultValue(
        structure.elementMultiplier,
        HeightmapTessellator.DEFAULT_STRUCTURE.elementMultiplier
    );
    const isBigEndian = defaultValue(
        structure.isBigEndian,
        HeightmapTessellator.DEFAULT_STRUCTURE.isBigEndian
    );

    let rectangleWidth = Rectangle.computeWidth(nativeRectangle);
    let rectangleHeight = Rectangle.computeHeight(nativeRectangle);

    const granularityX = rectangleWidth / (width - 1);
    const granularityY = rectangleHeight / (height - 1);

    if (!isGeographic) {
        rectangleWidth *= oneOverGlobeSemimajorAxis;
        rectangleHeight *= oneOverGlobeSemimajorAxis;
    }

    const radiiSquared = ellipsoid.radiiSquared;
    const radiiSquaredX = radiiSquared.x;
    const radiiSquaredY = radiiSquared.y;
    const radiiSquaredZ = radiiSquared.z;

    let minimumHeight = 65536.0;
    let maximumHeight = -65536.0;

    const fromENU = Transforms.eastNorthUpToFixedFrame(
        relativeToCenter,
        ellipsoid
    );
    const toENU = Matrix4.inverseTransformation(fromENU, matrix4Scratch);

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

    const minimum = minimumScratch;
    minimum.x = Number.POSITIVE_INFINITY;
    minimum.y = Number.POSITIVE_INFINITY;
    minimum.z = Number.POSITIVE_INFINITY;

    const maximum = maximumScratch;
    maximum.x = Number.NEGATIVE_INFINITY;
    maximum.y = Number.NEGATIVE_INFINITY;
    maximum.z = Number.NEGATIVE_INFINITY;

    let hMin = Number.POSITIVE_INFINITY;

    const gridVertexCount = width * height;
    const edgeVertexCount = skirtHeight > 0.0 ? width * 2 + height * 2 : 0;
    const vertexCount = gridVertexCount + edgeVertexCount;

    const positions = new Array(vertexCount);
    const heights = new Array(vertexCount);
    const uvs = new Array(vertexCount);
    const webMercatorTs = includeWebMercatorT ? new Array(vertexCount) : [];
    const geodeticSurfaceNormals = includeGeodeticSurfaceNormals
        ? new Array(vertexCount)
        : [];

    let startRow = 0;
    let endRow = height;
    let startCol = 0;
    let endCol = width;

    if (hasSkirts) {
        --startRow;
        ++endRow;
        --startCol;
        ++endCol;
    }

    const skirtOffsetPercentage = 0.00001;

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

        const isNorthEdge = rowIndex === startRow;
        const isSouthEdge = rowIndex === endRow - 1;
        if (skirtHeight > 0.0) {
        if (isNorthEdge) {
            latitude += skirtOffsetPercentage * rectangleHeight;
        } else if (isSouthEdge) {
            latitude -= skirtOffsetPercentage * rectangleHeight;
        }
        }

        const cosLatitude = cos(latitude);
        const nZ = sin(latitude);
        const kZ = radiiSquaredZ * nZ;

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

        const terrainOffset = row * (width * stride) + col * stride;

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
            const isWestEdge = colIndex === startCol;
            const isEastEdge = colIndex === endCol - 1;
            const isEdge = isNorthEdge || isSouthEdge || isWestEdge || isEastEdge;
            const isCorner =
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

        const nX = cosLatitude * cos(longitude);
        const nY = cosLatitude * sin(longitude);

        const kX = radiiSquaredX * nX;
        const kY = radiiSquaredY * nY;

        const gamma = sqrt(kX * nX + kY * nY + kZ * nZ);
        const oneOverGamma = 1.0 / gamma;

        const rSurfaceX = kX * oneOverGamma;
        const rSurfaceY = kY * oneOverGamma;
        const rSurfaceZ = kZ * oneOverGamma;

        const position = new Cartesian3();
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

    const boundingSphere3D = BoundingSphere.fromPoints(positions);
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
        const occluder = new EllipsoidalOccluder(ellipsoid);
        occludeePointInScaledSpace = occluder.computeHorizonCullingPointPossiblyUnderEllipsoid(
        relativeToCenter,
        positions,
        minimumHeight
        );
    }

    const aaBox = new AxisAlignedBoundingBox(minimum, maximum, relativeToCenter);
    const encoding = new TerrainEncoding(
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
    const vertices = new Float32Array(vertexCount * encoding.stride);

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
