use bevy::math::{DMat4, DVec2, DVec3};

use crate::{
    compress_texture_coordinates,
    geometry::AxisAlignedBoundingBox,
    math::{Cartesian3, Matrix4, SHIFT_LEFT_12},
    oct_pack_float,
    terrain_quantization::TerrainQuantization,
};
#[derive(Default, Clone, Debug, Copy)]
pub struct TerrainEncoding {
    pub quantization: TerrainQuantization,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub center: DVec3,
    pub to_scaled_enu: DMat4,
    pub from_scaled_enu: DMat4,
    pub matrix: DMat4,
    pub has_vertex_normals: bool,
    pub has_web_mercator_t: bool,
    pub has_geodetic_surface_normals: bool,
    pub exaggeration: f64,
    pub exaggeration_relative_height: f64,
    pub stride: u32,
    pub _offset_geodetic_surface_normal: f64,
    pub _offset_vertex_normal: f64,
}
impl TerrainEncoding {
    pub fn new(
        center: DVec3,
        axis_aligned_bounding_box_option: Option<AxisAlignedBoundingBox>,
        minimum_height_option: Option<f64>,
        maximum_height_option: Option<f64>,
        from_enu_option: Option<DMat4>,
        has_vertex_normals: bool,
        has_web_mercator_t_option: Option<bool>,
        has_geodetic_surface_normals_option: Option<bool>,
        exaggeration_option: Option<f64>,
        exaggeration_relative_height_option: Option<f64>,
    ) -> Self {
        let mut quantization = TerrainQuantization::NONE;
        let mut to_enu = DMat4::default();
        let mut matrix = DMat4::default();
        let mut axis_aligned_bounding_box: AxisAlignedBoundingBox =
            AxisAlignedBoundingBox::default();
        let mut minimum_height = 0.;
        let mut maximum_height = 0.;
        let mut from_enu = DMat4::default();

        if axis_aligned_bounding_box_option.is_some()
            && minimum_height_option.is_some()
            && maximum_height_option.is_some()
            && from_enu_option.is_some()
        {
            axis_aligned_bounding_box = axis_aligned_bounding_box_option.unwrap();
            minimum_height = minimum_height_option.unwrap();
            maximum_height = maximum_height_option.unwrap();
            from_enu = from_enu_option.unwrap();
            let minimum = axis_aligned_bounding_box.minimum;
            let maximum = axis_aligned_bounding_box.maximum;

            let dimensions = maximum - minimum;
            let h_dim = maximum_height - minimum_height;
            let max_dim = dimensions.maximum_component().max(h_dim);

            if max_dim < SHIFT_LEFT_12 - 1.0 {
                quantization = TerrainQuantization::BITS12;
                // quantization = TerrainQuantization::NONE;
            } else {
                quantization = TerrainQuantization::NONE;
            }

            to_enu = from_enu.inverse_transformation();

            let translation = minimum.negate();
            to_enu = DMat4::from_translation(translation) * to_enu;

            let mut scale = DVec3::ZERO;
            scale.x = 1.0 / dimensions.x;
            scale.y = 1.0 / dimensions.y;
            scale.z = 1.0 / dimensions.z;
            to_enu = DMat4::from_scale(scale) * to_enu;

            matrix = from_enu.clone();

            matrix.set_translation(&DVec3::ZERO);

            from_enu = from_enu.clone();

            let translation_matrix = DMat4::from_translation(minimum);
            let scale_matrix = DMat4::from_scale(dimensions);
            let st = translation_matrix * scale_matrix;
            from_enu = from_enu * st;
            matrix = matrix * st;
        }
        let mut encoding = Self {
            quantization,
            minimum_height: minimum_height,
            maximum_height: maximum_height,
            center,
            to_scaled_enu: to_enu,
            from_scaled_enu: from_enu,
            matrix,
            has_vertex_normals,
            has_web_mercator_t: has_web_mercator_t_option.unwrap_or(false),
            has_geodetic_surface_normals: has_geodetic_surface_normals_option.unwrap_or(false),
            exaggeration: exaggeration_option.unwrap_or(1.0),
            exaggeration_relative_height: exaggeration_relative_height_option.unwrap_or(0.0),
            stride: 0,
            _offset_geodetic_surface_normal: 0.0,
            _offset_vertex_normal: 0.0,
        };
        encoding._calculate_stride_and_offsets();
        return encoding;
    }
    pub fn _calculate_stride_and_offsets(&mut self) {
        let mut vertex_stride = 0;

        match self.quantization {
            TerrainQuantization::BITS12 => {
                vertex_stride += 3;
            }
            _ => {
                vertex_stride += 6;
            }
        }
        if self.has_web_mercator_t {
            vertex_stride += 1;
        }
        if self.has_vertex_normals {
            self._offset_vertex_normal = vertex_stride as f64;
            vertex_stride += 1;
        }
        if self.has_geodetic_surface_normals {
            self._offset_geodetic_surface_normal = vertex_stride as f64;
            vertex_stride += 3;
        }

        self.stride = vertex_stride;
    }
    pub fn encode(
        &self,
        vertex_buffer: &mut Vec<f32>,
        buffer_index: i64,
        position: &mut DVec3,
        uv: &DVec2,
        height: f64,
        normal_to_pack: Option<DVec2>,
        web_mercator_t: Option<f64>,
        geodetic_surface_normal: Option<&DVec3>,
    ) -> i64 {
        let u = uv.x;
        let v = uv.y;
        let mut new_buffer_index = buffer_index as usize;

        if self.quantization == TerrainQuantization::BITS12 {
            *position = self.to_scaled_enu.multiply_by_point(&position);

            position.x = position.x.clamp(0.0, 1.0);
            position.y = position.y.clamp(0.0, 1.0);
            position.z = position.z.clamp(0.0, 1.0);

            let h_dim = self.maximum_height - self.minimum_height;
            let h = ((height - self.minimum_height) / h_dim).clamp(0.0, 1.0);

            let mut cartesian2_scratch = DVec2::new(position.x, position.y);
            let compressed0 = compress_texture_coordinates(&cartesian2_scratch) as f32;

            cartesian2_scratch = DVec2::new(position.z, h);
            let compressed1 = compress_texture_coordinates(&cartesian2_scratch) as f32;

            cartesian2_scratch = DVec2::new(u, v);
            let compressed2 = compress_texture_coordinates(&cartesian2_scratch) as f32;

            vertex_buffer[new_buffer_index] = compressed0;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = compressed1;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = compressed2;
            new_buffer_index += 1;

            if self.has_web_mercator_t {
                let cartesian2_scratch = DVec2::new(web_mercator_t.unwrap(), 0.0);
                let compressed3 = compress_texture_coordinates(&cartesian2_scratch) as f32;
                vertex_buffer[new_buffer_index] = compressed3;
                new_buffer_index += 1;
            }
        } else {
            let cartesian3_scratch = position.subtract(self.center);
            // let cartesian3_scratch = position.clone();

            vertex_buffer[new_buffer_index] = cartesian3_scratch.x as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = cartesian3_scratch.y as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = cartesian3_scratch.z as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = height as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = u as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = v as f32;
            new_buffer_index += 1;

            if self.has_web_mercator_t {
                vertex_buffer[new_buffer_index] = web_mercator_t.unwrap() as f32;
                new_buffer_index += 1;
            }
        }

        if self.has_vertex_normals {
            vertex_buffer[new_buffer_index] = oct_pack_float(&normal_to_pack.unwrap()) as f32;
            new_buffer_index += 1;
        }

        if self.has_geodetic_surface_normals {
            let new_geodetic_surface_normal = geodetic_surface_normal.unwrap();
            vertex_buffer[new_buffer_index] = new_geodetic_surface_normal.x as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = new_geodetic_surface_normal.y as f32;
            new_buffer_index += 1;
            vertex_buffer[new_buffer_index] = new_geodetic_surface_normal.z as f32;
            new_buffer_index += 1;
        }

        return new_buffer_index as i64;
    }
    pub fn decode_height(&self, buffer: &Vec<f32>, index: usize) -> f64 {
        let index = index * self.stride as usize;
        return buffer[index + 3] as f64;
    }
}

#[cfg(test)]
mod tests {
    use crate::{decompress_texture_coordinates, Cartesian2};

    use super::*;

    #[test]
    fn test_1() {
        let coords = DVec2::new(1.0, 1.0);
        let compressed = compress_texture_coordinates(&coords);
        let decompressed = decompress_texture_coordinates(compressed);
        assert!(decompressed == coords);
    }
    #[test]
    fn test_2() {
        let coords = DVec2::new(0.5, 1.0);
        let compressed = compress_texture_coordinates(&coords);
        let decompressed = decompress_texture_coordinates(compressed);
        assert!(decompressed.equals_epsilon(coords, Some(1.0 / 4095.0), None));
    }
    #[test]
    fn test_3() {
        let coords = DVec2::new(1.0, 0.5);
        let compressed = compress_texture_coordinates(&coords);
        let decompressed = decompress_texture_coordinates(compressed);
        assert!(decompressed.equals_epsilon(coords, Some(1.0 / 4095.0), None));
    }
    #[test]
    fn test_4() {
        let coords = DVec2::new(0.99999999999999, 0.99999999999999);
        let compressed = compress_texture_coordinates(&coords);
        let decompressed = decompress_texture_coordinates(compressed);
        assert!(decompressed.equals_epsilon(coords, Some(1.0 / 4095.0), None));
    }
}
