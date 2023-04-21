pub trait ImageryProvider {
    fn requestImage(x: f64, y: f64, z: f64) -> Result<(), Box<dyn std::error::Error>>;
    fn ready()->bool{
        return true;
    }
}
