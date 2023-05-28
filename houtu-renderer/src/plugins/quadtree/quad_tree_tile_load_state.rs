#[derive(Debug, PartialEq)]
pub enum QuadtreeTileLoadState {
    START = 0,
    LOADING = 1,
    DONE = 2,
    FAILED = 3,
}
impl Default for QuadtreeTileLoadState {
    fn default() -> Self {
        Self::None
    }
}
