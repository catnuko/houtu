use bevy::prelude::*;

#[derive(Clone)]
pub struct AtlasAttachment {
    pub handle: Handle<Image>,
    pub translation_and_scale: Vec4,
    pub coordinate_rectangle: Vec4,
    pub web_mercator_t: f32,
    pub alpha: f32,
    pub day_alpha: f32,
    pub night_alpha: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub hue: f32,
    pub saturation: f32,
    pub one_over_gamma: f32,
    pub width: u32,
    pub height: u32,
}
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct ShaderDefines {
    pub apply_brightness: bool,
    pub apply_contrast: bool,
    pub apply_hue: bool,
    pub apply_saturation: bool,
    pub apply_gamma: bool,
    pub apply_alpha: bool,
    pub apply_day_night_alpha: bool,
    pub apply_split: bool,
    pub apply_cutout: bool,
    pub apply_color_to_alpha: bool,
    pub apply_quantization_bits12: bool,
    pub apply_webmercator_t: bool,
}
