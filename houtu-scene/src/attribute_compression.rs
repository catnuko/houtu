use bevy::math::DVec2;

pub fn compressTextureCoordinates(textureCoordinates: &DVec2) -> f64 {
    let x = maby(textureCoordinates.x, 0b0);
    let y = maby(textureCoordinates.y, 0b0);
    return 4096.0 * x + y;
}
pub fn maby(num: f64, num2: u64) -> f64 {
    let bits = num.to_bits(); // 将浮点数转换为u64类型的整数
    let result = bits | num2; // 对整数进行位运算操作
    let f_new = f64::from_bits(result); // 将结果转换回浮点数类型
    return f_new;
}
pub fn octPackFloat(encoded: &DVec2) -> f64 {
    return 256.0 * encoded.x + encoded.y;
}
