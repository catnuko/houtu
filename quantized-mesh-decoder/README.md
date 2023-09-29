# quantized-mesh-decoder
```rust
use quantized_mesh_decoder::{from_reader,QuantizedMeshTerrainData};
use std::{path::Path, fs::File};
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
let terrain_data = from_file("assets/tile-opentin.terrain");
```