use std::ops::Add;
#[derive(Debug, Copy)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Default for Cartographic {
    fn default() -> Self {
        return Cartographic::new(0.0, 0.0, 0.0);
    }
}
impl Clone for Cartographic {
    fn clone(&self) -> Self {
        return Cartographic::new(self.longitude, self.latitude, self.height);
    }
}
impl Cartographic {
    pub fn new(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude,
            latitude,
            height,
        }
    }
}
impl Add for Complex {
    type Output = Complex;

    fn add(self, other: Complex) -> Complex {
        Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }
}
