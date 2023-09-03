//https://github.com/CesiumGS/quantized-mesh
//https://blog.csdn.net/qgbihc/article/details/109207516
//https://www.cnblogs.com/oloroso/p/11080222.html#24%E6%89%A9%E5%B1%95%E6%95%B0%E6%8D%AE
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
pub struct VertexData {
    pub vertex_count: u32,
    pub u: Vec<u16>,      //u[vertexCount];      // 顶点横坐标
    pub v: Vec<u16>,      //v[vertexCount];      // 顶点纵坐标
    pub height: Vec<u16>, //height[vertexCount]; // 顶点高程值
}
pub struct IndexData16 {
    pub triangle_count: u32,
    pub indices: Vec<u16>, //indices[triangleCount * 3]; // 三角形顶点索引
}
pub struct IndexData32 {
    pub triangle_count: u32,
    pub indices: Vec<u32>, //indices[triangleCount * 3]; // 三角形顶点索引
}
pub enum TriangleIndices {
    IndexData16(IndexData16),
    IndexData32(IndexData32),
}
pub enum EdgeIndices {
    EdgeIndices16(EdgeIndices16),
    EdgeIndices32(EdgeIndices32),
}
pub struct EdgeIndices16 {
    pub west_vertex_count: u32,
    pub west_indices: Vec<u16>, //westIndices[westVertexCount];下面类似

    pub south_vertex_count: u32,
    pub south_indices: Vec<u16>,

    pub east_vertex_count: u32,
    pub east_indices: Vec<u16>,

    pub north_vertex_count: u32,
    pub north_indices: Vec<u16>,
}
pub struct EdgeIndices32 {
    pub west_vertex_count: u32,
    pub west_indices: Vec<u32>, //westIndices[westVertexCount];下面类似

    pub south_vertex_count: u32,
    pub south_indices: Vec<u32>,

    pub east_vertex_count: u32,
    pub east_indices: Vec<u32>,

    pub north_vertex_count: u32,
    pub north_indices: Vec<u32>,
}
pub struct ExtensionHeader {
    pub extension_id: u8,
    pub extension_length: u32,
}
pub struct OctEncodedVertexNormals {
    pub xy: Vec<u8>, //unsigned char xy[vertexCount * 2];
}
pub struct WaterMaskCovered {
    pub mask: u8, //A Terrain Tile covered entirely by land or water is defined by a single byte.
}
pub struct WaterMaskMix {
    pub mask: [u8; 65536], //A Terrain Tile containing a mix of land and water define a 256 x 256 grid of height values.
}
pub struct Metadata {
    pub json_length: u32,
    pub json: Vec<u8>,
}
pub enum WaterMask {
    WaterMaskCovered(WaterMaskCovered),
    WaterMaskMix(WaterMaskMix),
}
pub struct Extension {
    pub header: ExtensionHeader,
    pub vertex_normals: OctEncodedVertexNormals,
    pub water_mask: WaterMask,
    pub metadata: Metadata,
}
pub struct Terrain {
    pub header: QuantizedMeshHeader,
    pub vertex_data: Vec<i32>,
    pub triangle_indices: TriangleIndices,
    pub west_indices: EdgeIndices,
    pub north_indices: EdgeIndices,
    pub east_indices: EdgeIndices,
    pub south_indices: EdgeIndices,
    pub extension: Extension,
}
fn zigzag_decode(value: u16) -> i16 {
    (value >> 1) as i16 ^ -((value & 1) as i16)
}
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::Read;
pub fn from_file() {
    let file = File::open("./assets/tile-with-metadata-extension.terrain").unwrap();
    let _ = from_reader(file);
}
pub fn from_reader(mut rdr: impl Read) -> std::io::Result<()> {
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
    let mut vertex_data = vec![0_i16; (vertex_count * 3) as usize];
    let mut u = 0;
    let mut v = 0;
    let mut height = 0;
    for i in 0..vertex_count {
        u += zigzag_decode(rdr.read_u16::<LittleEndian>()?);
        v += zigzag_decode(rdr.read_u16::<LittleEndian>()?);
        height += zigzag_decode(rdr.read_u16::<LittleEndian>()?);
        vertex_data[i] = u;
        vertex_data[i + vertex_count] = v;
        vertex_data[i + vertex_count * 2] = height;
    }
    //TODO skip over any additional padding that was added for 2/4 byte alignment
    //if (position % bytesPerIndex !== 0) {
    //   position += bytesPerIndex - (position % bytesPerIndex)
    //}
    // decode triangle indices
    let triangle_count = rdr.read_u32::<LittleEndian>()?;
    let triangle_indices_count = triangle_count * 3;
    let triangle_indices: TriangleIndices;
    let use_32 = vertex_count > 65536; //64*1024
    if use_32 {
        let mut indices = vec![0u16; triangle_indices_count as usize];
        for i in 0..triangle_indices_count {
            let indice = rdr.read_u16::<LittleEndian>()?;
            indices[i as usize] = indice;

        }
        triangle_indices = TriangleIndices::IndexData16(IndexData16 {
            triangle_count: triangle_count,
            indices: indices,
        });
    } else {
        let mut indices = vec![0u32; triangle_indices_count as usize];
        for i in 0..triangle_indices_count {
            let indice = rdr.read_u32::<LittleEndian>()?;
            indices[i as usize] = indice;
        }
        triangle_indices = TriangleIndices::IndexData32(IndexData32 {
            triangle_count: triangle_count,
            indices: indices,
        });
    }
    // High water mark decoding based on decompressIndices_ in webgl-loader's loader.js.
    // https://code.google.com/p/webgl-loader/source/browse/trunk/samples/loader.js?r=99#55
    // Copyright 2012 Google Inc., Apache 2.0 license.
    let highest = 0;
    // for i in 0..triangle_count {
    //     let code = indices[i];
    //     indices[i] = highest - code;
    //     if (code == 0) {
    //         highest += 1;
    //     }
    // }

    println!("{}", header.horizon_occlusion_point_z);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        from_file()
    }
}
