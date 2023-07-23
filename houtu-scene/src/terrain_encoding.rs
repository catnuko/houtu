use bevy::math::{DMat4, DVec2, DVec3, DVec4};

use crate::{
    compressTextureCoordinates,
    geometry::AxisAlignedBoundingBox,
    math::{Cartesian3, Matrix4, SHIFT_LEFT_12},
    octPackFloat,
    terrain_quantization::TerrainQuantization,
};
#[derive(Default, Clone, Debug, Copy)]
pub struct TerrainEncoding {
    pub quantization: TerrainQuantization,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub center: DVec3,
    pub toScaledENU: DMat4,
    pub fromScaledENU: DMat4,
    pub matrix: DMat4,
    pub hasVertexNormals: bool,
    pub hasWebMercatorT: bool,
    pub hasGeodeticSurfaceNormals: bool,
    pub exaggeration: f64,
    pub exaggerationRelativeHeight: f64,
    pub stride: u32,
    pub _offsetGeodeticSurfaceNormal: f64,
    pub _offsetVertexNormal: f64,
}
impl TerrainEncoding {
    pub fn new(
        center: DVec3,
        axisAlignedBoundingBoxOption: Option<AxisAlignedBoundingBox>,
        minimumHeightOption: Option<f64>,
        maximumHeightOption: Option<f64>,
        fromENUOption: Option<DMat4>,
        hasVertexNormals: bool,
        hasWebMercatorTOption: Option<bool>,
        hasGeodeticSurfaceNormalsOption: Option<bool>,
        exaggerationOption: Option<f64>,
        exaggerationRelativeHeightOption: Option<f64>,
    ) -> Self {
        let mut quantization = TerrainQuantization::NONE;
        let mut toENU = DMat4::default();
        let mut matrix = DMat4::default();
        let mut axisAlignedBoundingBox: AxisAlignedBoundingBox = AxisAlignedBoundingBox::default();
        let mut minimum_height = 0.;
        let mut maximum_height = 0.;
        let mut fromENU = DMat4::default();

        if (axisAlignedBoundingBoxOption.is_some()
            && minimumHeightOption.is_some()
            && maximumHeightOption.is_some()
            && fromENUOption.is_some())
        {
            axisAlignedBoundingBox = axisAlignedBoundingBoxOption.unwrap();
            minimum_height = minimumHeightOption.unwrap();
            maximum_height = maximumHeightOption.unwrap();
            fromENU = fromENUOption.unwrap();
            let minimum = axisAlignedBoundingBox.minimum;
            let maximum = axisAlignedBoundingBox.maximum;

            let dimensions = maximum - minimum;
            let hDim = maximum_height - minimum_height;
            let maxDim = dimensions.maximum_component().max(hDim);

            if maxDim < SHIFT_LEFT_12 - 1.0 {
                quantization = TerrainQuantization::BITS12;
            } else {
                quantization = TerrainQuantization::NONE;
            }

            toENU = fromENU.inverse_transformation();

            let translation = minimum.negate();
            let mut toENU = DMat4::from_translation(translation) * toENU;

            let mut scale = DVec3::ZERO;
            scale.x = 1.0 / dimensions.x;
            scale.y = 1.0 / dimensions.y;
            scale.z = 1.0 / dimensions.z;
            toENU = DMat4::from_scale(scale) * toENU;

            matrix = fromENU.clone();

            matrix.set_translation(&DVec3::ZERO);

            fromENU = fromENU.clone();

            let translationMatrix = DMat4::from_translation(minimum);
            let scaleMatrix = DMat4::from_scale(dimensions);
            let st = translationMatrix * scaleMatrix;
            fromENU = fromENU * st;
            matrix = matrix * st;
        }
        let mut encoding = Self {
            quantization,
            minimum_height: minimum_height,
            maximum_height: maximum_height,
            center,
            toScaledENU: toENU,
            fromScaledENU: fromENU,
            matrix,
            hasVertexNormals,
            hasWebMercatorT: hasWebMercatorTOption.unwrap_or(false),
            hasGeodeticSurfaceNormals: hasGeodeticSurfaceNormalsOption.unwrap_or(false),
            exaggeration: exaggerationOption.unwrap_or(1.0),
            exaggerationRelativeHeight: exaggerationRelativeHeightOption.unwrap_or(0.0),
            stride: 0,
            _offsetGeodeticSurfaceNormal: 0.0,
            _offsetVertexNormal: 0.0,
        };
        encoding._calculateStrideAndOffsets();
        return encoding;
    }
    pub fn _calculateStrideAndOffsets(&mut self) {
        let mut vertex_stride = 0;

        match self.quantization {
            TerrainQuantization::BITS12 => {
                vertex_stride += 3;
            }
            _ => {
                vertex_stride += 6;
            }
        }
        if self.hasWebMercatorT {
            vertex_stride += 1;
        }
        if self.hasVertexNormals {
            self._offsetVertexNormal = vertex_stride as f64;
            vertex_stride += 1;
        }
        if self.hasGeodeticSurfaceNormals {
            self._offsetGeodeticSurfaceNormal = vertex_stride as f64;
            vertex_stride += 3;
        }

        self.stride = vertex_stride;
    }
    pub fn encode(
        &self,
        vertexBuffer: &mut Vec<f32>,
        bufferIndex: i64,
        position: &mut DVec3,
        uv: &DVec2,
        height: f64,
        normalToPack: Option<DVec2>,
        web_mercator_t: Option<f64>,
        geodeticSurfaceNormal: Option<&DVec3>,
    ) -> i64 {
        let u = uv.x;
        let v = uv.y;
        let mut new_bufferIndex = bufferIndex as usize;

        if self.quantization == TerrainQuantization::BITS12 {
            *position = self.toScaledENU.multiply_by_point(&position);

            position.x = position.x.clamp(0.0, 1.0);
            position.y = position.y.clamp(0.0, 1.0);
            position.z = position.z.clamp(0.0, 1.0);

            let hDim = self.maximum_height - self.minimum_height;
            let h = ((height - self.minimum_height) / hDim).clamp(0.0, 1.0);

            let mut cartesian2Scratch = DVec2::new(position.x, position.y);
            let compressed0 = compressTextureCoordinates(&cartesian2Scratch) as f32;

            cartesian2Scratch = DVec2::new(position.z, h);
            let compressed1 = compressTextureCoordinates(&cartesian2Scratch) as f32;

            cartesian2Scratch = DVec2::new(u, v);
            let compressed2 = compressTextureCoordinates(&cartesian2Scratch) as f32;

            vertexBuffer[new_bufferIndex] = compressed0;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = compressed1;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = compressed2;
            new_bufferIndex += 1;

            if self.hasWebMercatorT {
                let cartesian2Scratch = DVec2::new(web_mercator_t.unwrap(), 0.0);
                let compressed3 = compressTextureCoordinates(&cartesian2Scratch) as f32;
                vertexBuffer[new_bufferIndex] = compressed3;
                new_bufferIndex += 1;
            }
        } else {
            // let cartesian3Scratch = position.subtract(self.center);
            let cartesian3Scratch = position.clone();

            vertexBuffer[new_bufferIndex] = cartesian3Scratch.x as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = cartesian3Scratch.y as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = cartesian3Scratch.z as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = height as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = u as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = v as f32;
            new_bufferIndex += 1;

            if self.hasWebMercatorT {
                vertexBuffer[new_bufferIndex] = web_mercator_t.unwrap() as f32;
                new_bufferIndex += 1;
            }
        }

        if self.hasVertexNormals {
            vertexBuffer[new_bufferIndex] = octPackFloat(&normalToPack.unwrap()) as f32;
            new_bufferIndex += 1;
        }

        if self.hasGeodeticSurfaceNormals {
            let new_geodeticSurfaceNormal = geodeticSurfaceNormal.unwrap();
            vertexBuffer[new_bufferIndex] = new_geodeticSurfaceNormal.x as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = new_geodeticSurfaceNormal.y as f32;
            new_bufferIndex += 1;
            vertexBuffer[new_bufferIndex] = new_geodeticSurfaceNormal.z as f32;
            new_bufferIndex += 1;
        }

        return new_bufferIndex as i64;
    }
    pub fn decodeHeight(&self, buffer: &Vec<f32>, index: usize) -> f64 {
        let index = index * self.stride as usize;
        return buffer[index + 3] as f64;
    }
}
