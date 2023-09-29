//https://github.com/CesiumGS/quantized-mesh
//https://github.com/heremaps/quantized-mesh-decoder
//https://blog.csdn.net/qgbihc/article/details/109207516
//https://www.cnblogs.com/oloroso/p/11080222.html#24%E6%89%A9%E5%B1%95%E6%95%B0%E6%8D%AE
#[derive(Debug)]
pub struct QuantizedMeshHeader {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub minimum_height: f32,
    pub maximum_height: f32,
    pub bounding_sphere_center_x: f64,
    pub bounding_sphere_center_y: f64,
    pub bounding_sphere_center_z: f64,
    pub bounding_sphere_radius: f64,
    pub horizon_occlusion_point_x: f64,
    pub horizon_occlusion_point_y: f64,
    pub horizon_occlusion_point_z: f64,
}
#[derive(Debug)]
pub enum Indices {
    IndexData16(Vec<u16>),
    IndexData32(Vec<u32>),
}
impl Indices {
    pub fn len(&self) -> usize {
        match self {
            Indices::IndexData16(v) => return v.len(),
            Indices::IndexData32(v) => return v.len(),
        }
    }
}
#[derive(Default, Debug)]
pub struct Extension {
    pub vertex_normals: Option<Vec<u8>>,
    pub water_mask: Option<Vec<u8>>,
    pub metadata: Option<String>,
}
#[derive(Debug)]
pub struct QuantizedMeshTerrainData {
    pub header: QuantizedMeshHeader,
    pub vertex_data: Vec<u16>,
    pub triangle_indices: Indices,
    pub west_indices: Indices,
    pub north_indices: Indices,
    pub east_indices: Indices,
    pub south_indices: Indices,
    pub extension: Extension,
}
fn zigzag_decode(value: u16) -> i16 {
    (value >> 1) as i16 ^ -((value & 1) as i16)
}
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::vec;

const SIXTY_FOUR_KILOBYTES: u32 = 65536;
pub fn from_reader(mut rdr: impl Read) -> std::io::Result<QuantizedMeshTerrainData> {
    //decode header
    let center_x = rdr.read_f64::<LittleEndian>()?;
    let center_y = rdr.read_f64::<LittleEndian>()?;
    let center_z = rdr.read_f64::<LittleEndian>()?;
    let minimum_height = rdr.read_f32::<LittleEndian>()?;
    let maximum_height = rdr.read_f32::<LittleEndian>()?;
    let bounding_sphere_center_x = rdr.read_f64::<LittleEndian>()?;
    let bounding_sphere_center_y = rdr.read_f64::<LittleEndian>()?;
    let bounding_sphere_center_z = rdr.read_f64::<LittleEndian>()?;
    let bounding_sphere_radius = rdr.read_f64::<LittleEndian>()?;
    let horizon_occlusion_point_x = rdr.read_f64::<LittleEndian>()?;
    let horizon_occlusion_point_y = rdr.read_f64::<LittleEndian>()?;
    let horizon_occlusion_point_z = rdr.read_f64::<LittleEndian>()?;
    let header = QuantizedMeshHeader {
        center_x,
        center_y,
        center_z,
        maximum_height,
        minimum_height,
        bounding_sphere_center_x,
        bounding_sphere_center_y,
        bounding_sphere_center_z,
        bounding_sphere_radius,
        horizon_occlusion_point_x,
        horizon_occlusion_point_y,
        horizon_occlusion_point_z,
    };
    //decode vertex data
    let vertex_count = rdr.read_u32::<LittleEndian>()? as usize;
    let mut encoded_vertex_buffer = vec![0u16; vertex_count * 3];
    rdr.read_u16_into::<LittleEndian>(&mut encoded_vertex_buffer)?;
    let mut vertex_data = vec![0u16; vertex_count * 3];
    let mut u = 0;
    let mut v = 0;
    let mut height = 0;
    for i in 0..vertex_count {
        u += zigzag_decode(encoded_vertex_buffer[i]);
        v += zigzag_decode(encoded_vertex_buffer[i + vertex_count]);
        height += zigzag_decode(encoded_vertex_buffer[i + vertex_count * 2]);
        vertex_data[i] = u as u16;
        vertex_data[i + vertex_count] = v as u16;
        vertex_data[i + vertex_count * 2] = height as u16;
    }
    //TODO skip over any additional padding that was added for 2/4 byte alignment
    // if (position % bytesPerIndex !== 0) {
    //   position += bytesPerIndex - (position % bytesPerIndex)
    // }
    // decode triangle indices
    let triangle_count = rdr.read_u32::<LittleEndian>()?;
    let mut triangle_indices = create_typed_array_from_array_buffer(
        &mut rdr,
        vertex_count as u32,
        triangle_count as u32,
        (triangle_count * 3) as usize,
    )?;
    match triangle_indices {
        Indices::IndexData16(ref mut indices) => {
            // High water mark decoding based on decompressIndices_ in webgl-loader's loader.js.
            // https://code.google.com/p/webgl-loader/source/browse/trunk/samples/loader.js?r=99#55
            // Copyright 2012 Google Inc., Apache 2.0 license.
            let mut highest = 0;
            for i in 0..(triangle_count * 3) as usize {
                let code = indices[i];
                indices[i] = highest - code;
                if (code == 0) {
                    highest += 1;
                }
            }
        }
        Indices::IndexData32(ref mut indices) => {
            // High water mark decoding based on decompressIndices_ in webgl-loader's loader.js.
            // https://code.google.com/p/webgl-loader/source/browse/trunk/samples/loader.js?r=99#55
            // Copyright 2012 Google Inc., Apache 2.0 license.
            let mut highest = 0;
            for i in 0..(triangle_count * 3) as usize {
                let code = indices[i];
                indices[i] = highest - code;
                if (code == 0) {
                    highest += 1;
                }
            }
        }
    }
    let west_vertex_count = rdr.read_u32::<LittleEndian>()?;
    let west_indices = create_typed_array_from_array_buffer(
        &mut rdr,
        vertex_count as u32,
        west_vertex_count,
        west_vertex_count as usize,
    )?;
    let south_vertex_count = rdr.read_u32::<LittleEndian>()?;
    let south_indices = create_typed_array_from_array_buffer(
        &mut rdr,
        vertex_count as u32,
        south_vertex_count,
        south_vertex_count as usize,
    )?;
    let east_vertex_count = rdr.read_u32::<LittleEndian>()?;
    let east_indices = create_typed_array_from_array_buffer(
        &mut rdr,
        vertex_count as u32,
        east_vertex_count,
        east_vertex_count as usize,
    )?;

    let north_vertex_count = rdr.read_u32::<LittleEndian>()?;
    let north_indices = create_typed_array_from_array_buffer(
        &mut rdr,
        vertex_count as u32,
        north_vertex_count,
        north_vertex_count as usize,
    )?;
    let mut extension: Extension = Extension::default();
    while let Ok(extension_id) = rdr.read_u8() {
        let extension_length = rdr.read_u32::<LittleEndian>()?;
        match extension_id {
            //vertex normals
            1 => {
                let mut indices = vec![0u8; extension_length as usize];
                rdr.read_exact(&mut indices)?;
                extension.vertex_normals = Some(indices);
            }
            //water mask
            2 => {
                let mut indices = vec![0u8; extension_length as usize];
                rdr.read_exact(&mut indices)?;
                extension.water_mask = Some(indices);
            }
            //metadata
            4 => {
                let json_length = rdr.read_u32::<LittleEndian>()?;
                let mut json_buffer = vec![0u8; json_length as usize];
                rdr.read_exact(&mut json_buffer)?;
                if let Ok(json_str) = std::str::from_utf8(&json_buffer) {
                    let json_string: String = json_str.into();
                    extension.metadata = Some(json_string);
                };
            }
            _ => {
                panic!("error");
            }
        }
    }
    Ok(QuantizedMeshTerrainData {
        header,
        vertex_data,
        triangle_indices,
        west_indices,
        south_indices,
        east_indices,
        north_indices,
        extension,
    })
}
fn create_typed_array_from_array_buffer(
    rdr: &mut impl Read,
    vertex_count: u32,
    count: u32,
    length: usize,
) -> std::io::Result<Indices> {
    if vertex_count < SIXTY_FOUR_KILOBYTES {
        let mut indices = vec![0u16; length];
        rdr.read_u16_into::<LittleEndian>(&mut indices)?;
        return Ok(Indices::IndexData16(indices));
    } else {
        let mut indices = vec![0u32; length];
        rdr.read_u32_into::<LittleEndian>(&mut indices)?;
        return Ok(Indices::IndexData32(indices));
    }
}
#[cfg(test)]
mod tests {
    use std::{path::Path, fs::File};
    use super::*;
    pub fn from_file(path: &'static str) -> QuantizedMeshTerrainData {
        let path = Path::new(path);
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {:?}", why),
            Ok(file) => file,
        };
        match from_reader(file) {
            Ok(terrain) => {
                return terrain;
            }
            Err(error) => {
                panic!("error {:?}", error);
            }
        }
    }
    const VERTEX_DATA_VERTEX_COUNT: u32 = 4;
    const INDEX_DATA_TRIANGLE_COUNT: u32 = 2;

    const TRIANGLE_ONE: [[u16; 3]; 3] = [[8380, 26387, 0], [9841, 24918, 32767], [9841, 26387, 0]];
    const TRIANGLE_TWO: [[u16; 3]; 3] = [[9841, 24918, 32767], [8380, 26387, 0], [8380, 24918, 0]];
    const GROUND_TRUTH_TRIANGLES: [[[u16; 3]; 3]; 2] = [TRIANGLE_ONE, TRIANGLE_TWO];
    fn create_triangle(indices: &Vec<u32>, vertex_data: &Vec<u16>) -> Vec<[u16; 3]> {
        let vertex_count = (vertex_data.len() / 3) as u32;
        let mut triangle_list = vec![];
        for i in indices.iter() {
            let triangle = [
                vertex_data[(i + 0) as usize],
                vertex_data[(i + vertex_count) as usize],
                vertex_data[(i + vertex_count * 2) as usize],
            ];
            triangle_list.push(triangle);
        }
        return triangle_list;
    }
    fn compare_triangles(t1: &Vec<[u16; 3]>, t2: &[[u16; 3]; 3]) {
        for i in 0..3 {
            for j in 0..3 {
                assert!(t1[i][j] == t2[i][j]);
            }
        }
    }

    #[test]
    fn test_opentin() {
        let terrain_data = from_file("assets/tile-opentin.terrain");
        assert!(terrain_data.triangle_indices.len() == (INDEX_DATA_TRIANGLE_COUNT * 3) as usize);
        assert!(terrain_data.vertex_data.len() == (VERTEX_DATA_VERTEX_COUNT * 3) as usize);

        for i in 0..INDEX_DATA_TRIANGLE_COUNT {
            let index = i as usize;
            let inner_indices = match terrain_data.triangle_indices {
                Indices::IndexData16(ref v) => {
                    vec![
                        v[index * 3] as u32,
                        v[index * 3 + 1] as u32,
                        v[index * 3 + 2] as u32,
                    ]
                }
                Indices::IndexData32(ref v) => {
                    vec![v[index * 3], v[index * 3 + 1], v[index * 3 + 2]]
                }
            };
            let triangle = create_triangle(&inner_indices, &terrain_data.vertex_data);
            compare_triangles(&triangle, &GROUND_TRUTH_TRIANGLES[index])
        }
    }
    #[test]
    fn test_with_extension() {
        let terrain_data = from_file("assets/tile-with-extensions.terrain");
        assert!(terrain_data.extension.vertex_normals.is_some());
        assert!(terrain_data.extension.water_mask.is_some());
    }
    #[test]
    fn test_with_metadata_extension() {
        let terrain_data = from_file("assets/tile-with-metadata-extension.terrain");
        assert!(
            terrain_data.extension.metadata
                == Some(
                    "{\"geometricerror\":1232.3392654126055,\"surfacearea\":91962509942.00667}"
                        .into()
                )
        );
    }
}
