use bevy::{prelude::*, render::view::NoFrustumCulling};

use super::TileRendered;
#[derive(Bundle)]
pub struct TerrainBundle{
    tile_rendered:TileRendered,
    transform: Transform,
    global_transform: GlobalTransform,
    #[bundle]
    visibility_bundle: VisibilityBundle,
    no_frustum_culling: NoFrustumCulling,
}