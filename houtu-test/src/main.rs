use std::sync::RwLock;
fn main() {
    let mut v = vec![];
    v.push(1);
    println!("{}",v.len());
    v.push(2);
    println!("{}",v.len());
    v.push(3);
    println!("{}",v.len());
}
