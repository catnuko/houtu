use std::hash::Hash;

pub trait QuadtreeValue: PartialEq + Eq + Hash + Clone {
    // fn get_rect(&self) -> &Rect;
}
