use bevy::math::DVec2;

pub fn compress_texture_coordinates(texture_coordinates: &DVec2) -> f64 {
    // let x = bit_or(texture_coordinates.x * 4095.0, 0b0);
    // let y = bit_or(texture_coordinates.y * 4095.0, 0b0);
    let x = (texture_coordinates.x * 4095.0).floor();
    let y = (texture_coordinates.y * 4095.0).floor();
    return 4096.0 * x + y;
}
pub fn decompress_texture_coordinates(compressed: f64) -> DVec2 {
    let temp = compressed / 4096.0;
    let x_zero_to4095 = temp.floor();
    let x = x_zero_to4095 / 4095.0;
    let y = (compressed - x_zero_to4095 * 4096.0) / 4095.0;
    return DVec2::new(x, y);
}
pub fn oct_pack_float(encoded: &DVec2) -> f64 {
    return 256.0 * encoded.x + encoded.y;
}
