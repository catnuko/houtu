pub struct QuadtreePrimitiveDebug {
    pub enable_debug_output: bool,

    pub max_depth: u32,
    pub max_depth_visited: u32,
    pub tiles_visited: u32,
    pub tiles_culled: u32,
    pub tiles_rendered: u32,
    pub tiles_waiting_for_children: u32,

    pub last_max_depth: u32,
    pub last_max_depth_visited: u32,
    pub last_tiles_visited: u32,
    pub last_tiles_culled: u32,
    pub last_tiles_rendered: u32,
    pub last_tiles_waiting_for_children: u32,

    pub suspend_lod_update: bool,
}
impl QuadtreePrimitiveDebug {
    pub fn new() -> Self {
        Self {
            enable_debug_output: true,
            max_depth: 0,
            max_depth_visited: 0,
            tiles_visited: 0,
            tiles_culled: 0,
            tiles_rendered: 0,
            tiles_waiting_for_children: 0,
            last_max_depth: 0,
            last_max_depth_visited: 0,
            last_tiles_visited: 0,
            last_tiles_culled: 0,
            last_tiles_rendered: 0,
            last_tiles_waiting_for_children: 0,
            suspend_lod_update: false,
        }
    }
    pub fn reset(&mut self) {
        self.max_depth = 0;
        self.max_depth_visited = 0;
        self.tiles_visited = 0;
        self.tiles_culled = 0;
        self.tiles_rendered = 0;
        self.tiles_waiting_for_children = 0;
    }
}
