use bevy::prelude::*;
use houtu_scene::{
    Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme, Rectangle, TileBoundingRegion,
    TilingScheme, WebMercatorTilingScheme,
};

use crate::quadtree::tile_key::TileKey;

use super::{quadtree::Quadtree, tile_bounding_region::TileBoundingRegionCom};

/// add Rectangle and TileBoundingRegionCom
pub fn update_rectangle_system(
    mut commands: Commands,
    quadtree: Res<Quadtree>,
    tile_query: Query<(Entity, &TileKey), Without<Rectangle>>,
    ellipsoid: Res<Ellipsoid>,
) {
    for (entity, key) in tile_query.iter() {
        let rectangle = quadtree
            .tiling_scheme
            .tile_x_y_to_rectange(key.x, key.y, key.level);
        let tile_bounding_region = TileBoundingRegion::new(
            &rectangle,
            Some(0.0),
            Some(0.0),
            Some(&ellipsoid),
            Some(false),
        );
        commands
            .entity(entity)
            .insert((rectangle, TileBoundingRegionCom::new(tile_bounding_region)));
    }
}
