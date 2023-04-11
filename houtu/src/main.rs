use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    houtu::run();
    Ok(())
}
