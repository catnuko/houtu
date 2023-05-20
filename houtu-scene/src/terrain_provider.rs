use core::num;
use std::{collections::HashMap, f64::consts::PI, sync::Arc};

use bevy::prelude::In;

use crate::{Ellipsoid, FOUR_GIGABYTES, SIXTY_FOUR_KILOBYTES};
#[derive(Debug, Clone)]
pub struct IndicesAndEdges {
    pub indices: Vec<u32>,
    pub westIndicesSouthToNorth: Vec<u32>,
    pub southIndicesEastToWest: Vec<u32>,
    pub eastIndicesNorthToSouth: Vec<u32>,
    pub northIndicesWestToEast: Vec<u32>,
    pub indexCountWithoutSkirts: Option<u32>,
}
pub struct InnerIndicesAndEdges {
    pub westIndicesSouthToNorth: Vec<u32>,
    pub southIndicesEastToWest: Vec<u32>,
    pub eastIndicesNorthToSouth: Vec<u32>,
    pub northIndicesWestToEast: Vec<u32>,
}
const heightmapTerrainQuality: f64 = 0.25;
// static mut regularGridAndEdgeIndicesCache: Vec<Vec<IndicesAndEdges>> = Vec::new();
// static mut regularGridAndSkirtAndEdgeIndicesCache: Vec<Vec<IndicesAndEdges>> = Vec::new();
// static mut regularGridIndicesCache: Vec<Vec<Vec<u32>>> = Vec::new();
pub struct IndicesAndEdgesCache {
    pub regularGridAndEdgeIndicesCache: HashMap<String, IndicesAndEdges>,
    pub regularGridAndSkirtAndEdgeIndicesCache: HashMap<String, IndicesAndEdges>,
    pub regularGridIndicesCache: HashMap<String, Vec<u32>>,
}
impl IndicesAndEdgesCache {
    pub fn new() -> Self {
        Self {
            regularGridAndEdgeIndicesCache: HashMap::new(),
            regularGridAndSkirtAndEdgeIndicesCache: HashMap::new(),
            regularGridIndicesCache: HashMap::new(),
        }
    }
    fn get_key(width: u32, height: u32) -> String {
        format!("{}-{}", width, height)
    }
    pub fn getRegularGridIndices(&mut self, width: u32, height: u32) -> Vec<u32> {
        if ((width * height) as u64 >= FOUR_GIGABYTES) {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self.regularGridIndicesCache.get(key.as_str()).cloned();

        if value.is_none() {
            let mut new_value: Vec<u32> = vec![0; ((width - 1) * (height - 1) * 6) as usize];
            addRegularGridIndices(width, height, &mut new_value, 0);
            self.regularGridIndicesCache.insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }
    }
    pub fn getRegularGridAndSkirtIndicesAndEdgeIndices(
        &mut self,
        width: u32,
        height: u32,
    ) -> IndicesAndEdges {
        if ((width * height) as u64 >= FOUR_GIGABYTES) {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self
            .regularGridAndSkirtAndEdgeIndicesCache
            .get(key.as_str())
            .cloned();

        if value.is_none() {
            let gridVertexCount = width * height;
            let mut gridIndexCount = (width - 1) * (height - 1) * 6;
            let edgeVertexCount = width * 2 + height * 2;
            let edgeIndexCount = 0.max(edgeVertexCount - 4) * 6;
            let vertexCount = gridVertexCount + edgeVertexCount;
            let indexCount = gridIndexCount + edgeIndexCount;

            let edgeIndices = getEdgeIndices(width, height);
            let westIndicesSouthToNorth = edgeIndices.westIndicesSouthToNorth;
            let southIndicesEastToWest = edgeIndices.southIndicesEastToWest;
            let eastIndicesNorthToSouth = edgeIndices.eastIndicesNorthToSouth;
            let northIndicesWestToEast = edgeIndices.northIndicesWestToEast;

            let mut indices: Vec<u32> = vec![0; indexCount as usize];
            addRegularGridIndices(width, height, &mut indices, 0);
            gridIndexCount = addSkirtIndices(
                &westIndicesSouthToNorth,
                &southIndicesEastToWest,
                &eastIndicesNorthToSouth,
                &northIndicesWestToEast,
                gridVertexCount,
                &mut indices,
                gridIndexCount,
            );
            let new_value = IndicesAndEdges {
                indices: indices,
                westIndicesSouthToNorth: westIndicesSouthToNorth,
                southIndicesEastToWest: southIndicesEastToWest,
                eastIndicesNorthToSouth: eastIndicesNorthToSouth,
                northIndicesWestToEast: northIndicesWestToEast,
                indexCountWithoutSkirts: Some(gridIndexCount),
            };
            self.regularGridAndSkirtAndEdgeIndicesCache
                .insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }
    }
    pub fn getRegularGridIndicesAndEdgeIndices(
        &mut self,
        width: u32,
        height: u32,
    ) -> IndicesAndEdges {
        if ((width * height) as u64 >= FOUR_GIGABYTES) {
            panic!(
                "The total number of vertices (width * height) must be less than 4,294,967,296."
            );
        }
        let key = Self::get_key(width, height);
        let value = self
            .regularGridAndEdgeIndicesCache
            .get(key.as_str())
            .cloned();

        if value.is_none() {
            let indices = self.getRegularGridIndices(width, height);

            let edgeIndices = getEdgeIndices(width, height);
            let westIndicesSouthToNorth = edgeIndices.westIndicesSouthToNorth;
            let southIndicesEastToWest = edgeIndices.southIndicesEastToWest;
            let eastIndicesNorthToSouth = edgeIndices.eastIndicesNorthToSouth;
            let northIndicesWestToEast = edgeIndices.northIndicesWestToEast;

            let new_value = IndicesAndEdges {
                indices: indices,
                westIndicesSouthToNorth: westIndicesSouthToNorth,
                southIndicesEastToWest: southIndicesEastToWest,
                eastIndicesNorthToSouth: eastIndicesNorthToSouth,
                northIndicesWestToEast: northIndicesWestToEast,
                indexCountWithoutSkirts: None,
            };
            self.regularGridAndEdgeIndicesCache
                .insert(key, new_value.clone());
            return new_value;
        } else {
            return value.unwrap();
        }

        pub fn getEstimatedLevelZeroGeometricErrorForAHeightmap(
            ellipsoid: &Ellipsoid,
            tileImageWidth: u32,
            numberOfTilesAtLevelZero: u32,
        ) -> f64 {
            return getEstimatedLevelZeroGeometricErrorForAHeightmap(
                ellipsoid,
                tileImageWidth,
                numberOfTilesAtLevelZero,
            );
        }
    }
}

pub fn getEstimatedLevelZeroGeometricErrorForAHeightmap(
    ellipsoid: &Ellipsoid,
    tileImageWidth: u32,
    numberOfTilesAtLevelZero: u32,
) -> f64 {
    return ((ellipsoid.maximumRadius * 2. * PI * heightmapTerrainQuality)
        / (tileImageWidth * numberOfTilesAtLevelZero) as f64);
}
// pub fn getRegularGridIndices(width: u32, height: u32) -> Vec<u32> {
//     if (width * height >= FOUR_GIGABYTES) {
//         panic!("The total number of vertices (width * height) must be less than 4,294,967,296.");
//     }

//     //>>includeEnd('debug');
//     let byWidth: &Vec<Vec<u32>> = {
//         let byWidthOption = unsafe {
//             regularGridIndicesCache.get(width as usize);
//         };
//         if byWidthOption.is_none() {
//             let value = Vec::<Vec<u32>>::new();
//             unsafe {
//                 regularGridIndicesCache[width as usize] = value;
//             }
//             &value
//         } else {
//             &byWidthOption.unwrap()
//         }
//     };
//     let indices: Vec<u32> = {
//         let indicesAndEdgesOption = byWidth.get_mut(height as usize);
//         if indicesAndEdgesOption.is_none() {
//             let value: Vec<u32> = vec![0; ((width - 1) * (height - 1) * 6) as usize];
//             byWidth[height as usize] = value;
//             addRegularGridIndices(width, height, &mut value, 0);
//             value
//         } else {
//             indicesAndEdgesOption.unwrap().clone()
//         }
//     };
//     return indices;
// }
// pub fn getRegularGridAndSkirtIndicesAndEdgeIndices(width: u32, height: u32) -> IndicesAndEdges {
//     if (width * height >= FOUR_GIGABYTES) {
//         panic!("The total number of vertices (width * height) must be less than 4,294,967,296.");
//     }
//     //>>includeEnd('debug');
//     let byWidth: &Vec<IndicesAndEdges> = {
//         let byWidthOption = regularGridAndSkirtAndEdgeIndicesCache.get_mut(width as usize);
//         if byWidthOption.is_none() {
//             let value = Vec::<IndicesAndEdges>::new();
//             regularGridAndSkirtAndEdgeIndicesCache[width as usize] = value;
//             &value
//         } else {
//             &byWidthOption.unwrap()
//         }
//     };

//     let indicesAndEdges: IndicesAndEdges = {
//         let indicesAndEdgesOption = byWidth.get_mut(height as usize);
//         if indicesAndEdgesOption.is_none() {
//             let gridVertexCount = width * height;
//             let gridIndexCount = (width - 1) * (height - 1) * 6;
//             let edgeVertexCount = width * 2 + height * 2;
//             let edgeIndexCount = 0.max(edgeVertexCount - 4) * 6;
//             let vertexCount = gridVertexCount + edgeVertexCount;
//             let indexCount = gridIndexCount + edgeIndexCount;

//             let edgeIndices = getEdgeIndices(width, height);
//             let westIndicesSouthToNorth = edgeIndices.westIndicesSouthToNorth;
//             let southIndicesEastToWest = edgeIndices.southIndicesEastToWest;
//             let eastIndicesNorthToSouth = edgeIndices.eastIndicesNorthToSouth;
//             let northIndicesWestToEast = edgeIndices.northIndicesWestToEast;

//             let indices = Vec::<u32>::new();
//             addRegularGridIndices(width, height, &mut indices, 0);
//             addSkirtIndices(
//                 &westIndicesSouthToNorth,
//                 &southIndicesEastToWest,
//                 &eastIndicesNorthToSouth,
//                 &northIndicesWestToEast,
//                 gridVertexCount,
//                 &mut indices,
//                 gridIndexCount,
//             );
//             let value = IndicesAndEdges {
//                 indices: indices,
//                 westIndicesSouthToNorth: westIndicesSouthToNorth,
//                 southIndicesEastToWest: southIndicesEastToWest,
//                 eastIndicesNorthToSouth: eastIndicesNorthToSouth,
//                 northIndicesWestToEast: northIndicesWestToEast,
//                 indexCountWithoutSkirts: Some(gridIndexCount),
//             };
//             byWidth[height as usize] = value;
//             value
//         } else {
//             indicesAndEdgesOption.unwrap().clone()
//         }
//     };
//     return indicesAndEdges;
// }
// pub fn getRegularGridIndicesAndEdgeIndices(width: u32, height: u32) -> IndicesAndEdges {
//     if (width * height >= FOUR_GIGABYTES) {
//         panic!("The total number of vertices (width * height) must be less than 4,294,967,296.");
//     }
//     //>>includeEnd('debug');
//     let byWidth: &Vec<IndicesAndEdges> = {
//         let byWidthOption = regularGridAndEdgeIndicesCache.get_mut(width as usize);
//         if byWidthOption.is_none() {
//             let value = Vec::<IndicesAndEdges>::new();
//             regularGridAndEdgeIndicesCache[width as usize] = value;
//             &value
//         } else {
//             &byWidthOption.unwrap()
//         }
//     };

//     let indicesAndEdges: IndicesAndEdges = {
//         let indicesAndEdgesOption = byWidth.get_mut(height as usize);
//         if indicesAndEdgesOption.is_none() {
//             let indices = getRegularGridIndices(width, height);

//             let edgeIndices = getEdgeIndices(width, height);
//             let westIndicesSouthToNorth = edgeIndices.westIndicesSouthToNorth;
//             let southIndicesEastToWest = edgeIndices.southIndicesEastToWest;
//             let eastIndicesNorthToSouth = edgeIndices.eastIndicesNorthToSouth;
//             let northIndicesWestToEast = edgeIndices.northIndicesWestToEast;

//             let value = IndicesAndEdges {
//                 indices: indices,
//                 westIndicesSouthToNorth: westIndicesSouthToNorth,
//                 southIndicesEastToWest: southIndicesEastToWest,
//                 eastIndicesNorthToSouth: eastIndicesNorthToSouth,
//                 northIndicesWestToEast: northIndicesWestToEast,
//                 indexCountWithoutSkirts: None,
//             };
//             byWidth[height as usize] = value;
//             value
//         } else {
//             indicesAndEdgesOption.unwrap().clone()
//         }
//     };
//     return indicesAndEdges;
// }

pub fn getEdgeIndices(width: u32, height: u32) -> InnerIndicesAndEdges {
    let mut westIndicesSouthToNorth = vec![0; height as usize];
    let mut southIndicesEastToWest = vec![0; width as usize];
    let mut eastIndicesNorthToSouth = vec![0; height as usize];
    let mut northIndicesWestToEast = vec![0; width as usize];

    for i in 0..width {
        let ii = i as usize;
        northIndicesWestToEast[ii] = i;
        southIndicesEastToWest[ii] = width * height - 1 - i;
    }
    for i in 0..height {
        let ii = i as usize;
        eastIndicesNorthToSouth[ii] = (i + 1) * width - 1;
        westIndicesSouthToNorth[ii] = (height - i - 1) * width;
    }

    return InnerIndicesAndEdges {
        westIndicesSouthToNorth,
        southIndicesEastToWest,
        eastIndicesNorthToSouth,
        northIndicesWestToEast,
    };
}
pub fn addSkirtIndices(
    westIndicesSouthToNorth: &Vec<u32>,
    southIndicesEastToWest: &Vec<u32>,
    eastIndicesNorthToSouth: &Vec<u32>,
    northIndicesWestToEast: &Vec<u32>,
    vertexCount: u32,
    indices: &mut Vec<u32>,
    offset: u32,
) -> u32 {
    let mut offset = offset;
    let mut vertexIndex = vertexCount;
    offset = inner_addSkirtIndices(westIndicesSouthToNorth, vertexIndex, indices, offset);
    vertexIndex += westIndicesSouthToNorth.len() as u32;
    offset = inner_addSkirtIndices(southIndicesEastToWest, vertexIndex, indices, offset);
    vertexIndex += southIndicesEastToWest.len() as u32;
    offset = inner_addSkirtIndices(eastIndicesNorthToSouth, vertexIndex, indices, offset);
    vertexIndex += eastIndicesNorthToSouth.len() as u32;
    return inner_addSkirtIndices(northIndicesWestToEast, vertexIndex, indices, offset);
}
fn inner_addSkirtIndices(
    edgeIndices: &Vec<u32>,
    vertexIndex: u32,
    indices: &mut Vec<u32>,
    offset: u32,
) -> u32 {
    let mut previousIndex = edgeIndices[0];
    let length = edgeIndices.len();
    let mut uoffset = offset as usize;
    let mut vertexIndex = vertexIndex;
    for i in 1..length {
        let index = edgeIndices[i];

        indices[uoffset] = previousIndex;
        uoffset += 1;
        indices[uoffset] = index;
        uoffset += 1;
        indices[uoffset] = vertexIndex;
        uoffset += 1;

        indices[uoffset] = vertexIndex;
        uoffset += 1;
        indices[uoffset] = index;
        uoffset += 1;
        indices[uoffset] = vertexIndex + 1;
        uoffset += 1;

        previousIndex = index;
        vertexIndex += 1;
    }

    return uoffset as u32;
}
pub fn addRegularGridIndices(width: u32, height: u32, indices: &mut Vec<u32>, offset: u32) {
    let mut index = 0;
    let mut uoffset = offset as usize;
    for j in 0..(height - 1) {
        for i in 0..(width - 1) {
            let upperLeft = index;
            let lowerLeft = upperLeft + width;
            let lowerRight = lowerLeft + 1;
            let upperRight = upperLeft + 1;

            indices[uoffset] = upperLeft;
            uoffset += 1;
            indices[uoffset] = lowerLeft;
            uoffset += 1;
            indices[uoffset] = upperRight;
            uoffset += 1;
            indices[uoffset] = upperRight;
            uoffset += 1;
            indices[uoffset] = lowerLeft;
            uoffset += 1;
            indices[uoffset] = lowerRight;
            uoffset += 1;

            index += 1;
        }
        index += 1;
    }
}
